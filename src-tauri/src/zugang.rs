//! Wie Horaka Desktop sich ausweist: über OAuth (Anmeldung im Browser,
//! Zugangstoken für eine Stunde, Refresh-Token mit Rotation) oder mit einem
//! eingefügten Gerätetoken. Gespeichert als JSON im Schlüsselbund des
//! Systems (Windows Credential Manager, macOS Keychain), nie in einer Datei,
//! und im Speicher gehalten, damit der Abgleich nicht alle 30 Sekunden den
//! Schlüsselbund fragt.

use crate::api::Fehler;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

const DIENST: &str = "com.horaka.desktop";
const KONTO: &str = "zugang";

/// So viele Sekunden vor dem Ablauf wird ein Zugangstoken erneuert.
const VORLAUF: i64 = 120;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "art", rename_all = "snake_case")]
pub enum Zugang {
    Token {
        token: String,
    },
    Oauth {
        client_id: String,
        access: String,
        refresh: String,
        /// Ablauf des Zugangstokens als Unix-Zeit.
        ablauf: i64,
        /// Bewilligte Rechte (Scopes), durch Leerzeichen getrennt.
        #[serde(default)]
        rechte: String,
    },
}

impl Zugang {
    fn access(&self) -> &str {
        match self {
            Zugang::Token { token } => token,
            Zugang::Oauth { access, .. } => access,
        }
    }
}

static ZWISCHEN: Mutex<Option<Zugang>> = Mutex::new(None);

/// Nur einer erneuert: mit Rotation wäre ein zweiter Versuch mit demselben
/// Refresh-Token ungültig und würde die Verbindung trennen.
static ERNEUERN: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn eintrag() -> Result<keyring::Entry, Fehler> {
    keyring::Entry::new(DIENST, KONTO).map_err(|e| Fehler::neu("keychain", Some(e.to_string())))
}

pub fn lesen() -> Option<Zugang> {
    let mut zwischen = ZWISCHEN.lock().unwrap();
    if zwischen.is_none() {
        let json = eintrag().ok()?.get_password().ok()?;
        *zwischen = serde_json::from_str(&json).ok();
    }
    zwischen.clone()
}

pub fn vorhanden() -> bool {
    lesen().is_some()
}

pub fn ist_oauth() -> bool {
    matches!(lesen(), Some(Zugang::Oauth { .. }))
}

pub fn speichern(zugang: Zugang) -> Result<(), Fehler> {
    let json = serde_json::to_string(&zugang).expect("Zugang als JSON");
    eintrag()?
        .set_password(&json)
        .map_err(|e| Fehler::neu("keychain", Some(e.to_string())))?;
    *ZWISCHEN.lock().unwrap() = Some(zugang);
    Ok(())
}

/// Löscht den Zugang; bei OAuth wird der Refresh-Token auch in Horaka
/// widerrufen (ohne Netz bleibt er bis zum Ablauf gültig).
pub fn loeschen() {
    let alt = ZWISCHEN.lock().unwrap().take();
    if let Ok(e) = eintrag() {
        let _ = e.delete_credential();
    }
    if let Some(Zugang::Oauth {
        client_id, refresh, ..
    }) = alt
    {
        tauri::async_runtime::spawn(async move {
            crate::oauth::widerrufen(&client_id, &refresh).await;
        });
    }
}

pub fn jetzt() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Ein Zugangstoken, der gilt. `abgelehnt` ist der Token, den die API
/// gerade mit 401 abgewiesen hat: dann wird erneuert, auch wenn er laut
/// Uhr noch gälte, außer jemand anderes hat inzwischen schon erneuert.
pub async fn gueltiger_token(abgelehnt: Option<&str>) -> Result<String, Fehler> {
    let zugang = lesen().ok_or_else(|| Fehler::neu("unauthorized", None))?;

    let Zugang::Oauth { ablauf, .. } = &zugang else {
        return Ok(zugang.access().to_string());
    };
    if abgelehnt.is_none() && *ablauf - jetzt() > VORLAUF {
        return Ok(zugang.access().to_string());
    }

    let _sperre = ERNEUERN.lock().await;
    let zugang = lesen().ok_or_else(|| Fehler::neu("unauthorized", None))?;
    let Zugang::Oauth {
        client_id,
        access,
        refresh,
        ablauf,
        ..
    } = zugang
    else {
        return Ok(zugang.access().to_string());
    };

    let schon_erneuert = match abgelehnt {
        Some(alt) => alt != access,
        None => ablauf - jetzt() > VORLAUF,
    };
    if schon_erneuert {
        return Ok(access);
    }

    match crate::oauth::erneuern(&client_id, &refresh).await {
        Ok(neu) => {
            let access = neu.access().to_string();
            speichern(neu)?;
            Ok(access)
        }
        // Ohne Netz bleibt der Zugang; abgelehnt heißt: neu anmelden.
        Err(f) if f.code == "network" => Err(f),
        Err(_) => Err(Fehler::neu("unauthorized", None)),
    }
}
