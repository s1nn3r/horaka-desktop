//! Zugang zur REST-API von Horaka (https://api.horaka.com/v1). Nur die
//! Trigger-Endpunkte: `GET /track/choices`, `GET /timer` und
//! `POST /track/{start,stop,switch}` mit Idempotency-Key und
//! `PATCH /track/entries/{id}` für die Notiz. Woher der Token
//! kommt (OAuth oder Gerätetoken) und wie er erneuert wird, regelt
//! `zugang.rs`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::Duration;

/// Adresse der API, für Tests überschreibbar mit `HORAKA_API_URL`.
pub fn basis() -> String {
    std::env::var("HORAKA_API_URL")
        .unwrap_or_else(|_| "https://api.horaka.com/v1".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Fehler für die Oberfläche: `code` aus der API (`unauthorized`,
/// `forbidden`, `project_not_allowed` …) oder eigene (`network`,
/// `keychain`), `detail` ist die Meldung der API in der Sprache des Kontos.
#[derive(Debug, Clone, Serialize)]
pub struct Fehler {
    pub code: String,
    pub detail: Option<String>,
}

impl Fehler {
    pub fn neu(code: &str, detail: Option<String>) -> Self {
        Self {
            code: code.to_string(),
            detail,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Eintrag {
    pub id: i64,
    pub project_id: i64,
    pub activity_id: Option<i64>,
    pub started_at: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timer {
    pub running: bool,
    pub count: i64,
    pub elapsed_seconds: Option<i64>,
    pub entry: Option<Eintrag>,
    pub entries: Vec<Eintrag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taetigkeit {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projekt {
    pub id: i64,
    pub name: String,
    pub client_id: i64,
    pub client_name: String,
    pub color: String,
    pub status: String,
    pub activities: Vec<Taetigkeit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kombination {
    pub project_id: i64,
    pub activity_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auswahl {
    pub projects: Vec<Projekt>,
    pub favorites: Vec<Kombination>,
    pub recent: Vec<Kombination>,
    pub parallel_timers: bool,
}

impl Auswahl {
    pub fn projekt(&self, id: i64) -> Option<&Projekt> {
        self.projects.iter().find(|p| p.id == id)
    }

    /// „Kunde · Projekt“ und, falls vorhanden, die Tätigkeit dahinter.
    pub fn bezeichnung(&self, projekt_id: i64, taetigkeit_id: Option<i64>) -> String {
        let Some(projekt) = self.projekt(projekt_id) else {
            return format!("#{projekt_id}");
        };
        let mut text = format!("{} · {}", projekt.client_name, projekt.name);
        if let Some(t) = taetigkeit_id.and_then(|id| projekt.activities.iter().find(|a| a.id == id))
        {
            text.push_str(&format!(" · {}", t.name));
        }
        text
    }
}

#[derive(Deserialize)]
struct Huelle<T> {
    data: T,
}

#[derive(Deserialize)]
struct FehlerHuelle {
    errors: Vec<FehlerEintrag>,
}

#[derive(Deserialize)]
struct FehlerEintrag {
    code: Option<String>,
    title: Option<String>,
    detail: Option<String>,
}

pub fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("HorakaDesktop/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(15))
        .build()
        .expect("HTTP-Client")
}

/// Sprachen der App (Fenster, Tray, Meldungen).
pub const SPRACHEN: [&str; 8] = ["de", "en", "fr", "es", "it", "nl", "pl", "pt"];

/// Sprache aus den Einstellungen; `system` (oder leer) heißt Betriebssystem.
static WAHL: Mutex<String> = Mutex::new(String::new());

pub fn sprache_waehlen(wahl: &str) {
    if let Ok(mut w) = WAHL.lock() {
        *w = wahl.to_string();
    }
}

/// Sprache der App und der Meldungen der API: gewählt, sonst die des
/// Systems, sonst Englisch.
pub fn sprache() -> &'static str {
    let wahl = WAHL.lock().map(|w| w.clone()).unwrap_or_default();
    let gesucht = if wahl.is_empty() || wahl == "system" {
        sys_locale::get_locale().unwrap_or_default().to_lowercase()
    } else {
        wahl
    };
    SPRACHEN
        .into_iter()
        .find(|code| gesucht.starts_with(code))
        .unwrap_or("en")
}

async fn anfrage<T: DeserializeOwned>(
    methode: reqwest::Method,
    pfad: &str,
    token: &str,
    rumpf: Option<serde_json::Value>,
) -> Result<T, Fehler> {
    let mut anfrage = http()
        .request(methode.clone(), format!("{}/{}", basis(), pfad))
        .bearer_auth(token)
        .header("Accept", "application/json")
        .header("Accept-Language", sprache());

    if methode == reqwest::Method::POST {
        anfrage = anfrage.header("Idempotency-Key", uuid::Uuid::new_v4().to_string());
    }
    if let Some(r) = rumpf {
        anfrage = anfrage.json(&r);
    }

    let antwort = anfrage
        .send()
        .await
        .map_err(|e| Fehler::neu("network", Some(e.to_string())))?;
    let status = antwort.status();

    if status.is_success() {
        return antwort
            .json::<Huelle<T>>()
            .await
            .map(|h| h.data)
            .map_err(|e| Fehler::neu("invalid_response", Some(e.to_string())));
    }

    let erster = antwort
        .json::<FehlerHuelle>()
        .await
        .ok()
        .and_then(|h| h.errors.into_iter().next());
    let code = match status.as_u16() {
        401 => "unauthorized".to_string(),
        429 => "rate_limited".to_string(),
        _ => erster
            .as_ref()
            .and_then(|e| e.code.clone())
            .unwrap_or_else(|| format!("http_{}", status.as_u16())),
    };

    Err(Fehler::neu(
        &code,
        erster.and_then(|e| e.detail.or(e.title)),
    ))
}

/// Ein Aufruf mit gültigem Token. Lehnt die API einen OAuth-Token ab
/// (abgelaufen oder widerrufen), wird er einmal erneuert und der Aufruf
/// wiederholt; scheitert auch das, ist die Verbindung weg.
async fn aufrufen<T: DeserializeOwned>(
    methode: reqwest::Method,
    pfad: &str,
    rumpf: Option<serde_json::Value>,
) -> Result<T, Fehler> {
    let token = crate::zugang::gueltiger_token(None).await?;
    match anfrage(methode.clone(), pfad, &token, rumpf.clone()).await {
        Err(f) if f.code == "unauthorized" && crate::zugang::ist_oauth() => {
            let neu = crate::zugang::gueltiger_token(Some(&token)).await?;
            anfrage(methode, pfad, &neu, rumpf).await
        }
        ergebnis => ergebnis,
    }
}

pub async fn auswahl_mit(token: &str) -> Result<Auswahl, Fehler> {
    anfrage(reqwest::Method::GET, "track/choices", token, None).await
}

pub async fn auswahl() -> Result<Auswahl, Fehler> {
    aufrufen(reqwest::Method::GET, "track/choices", None).await
}

pub async fn timer() -> Result<Timer, Fehler> {
    aufrufen(reqwest::Method::GET, "timer", None).await
}

/// Notiz eines eigenen Eintrags (laufend oder bis 24 Stunden nach dem
/// Ende); `None` löscht sie.
pub async fn notiz_setzen(id: i64, notiz: Option<String>) -> Result<Eintrag, Fehler> {
    aufrufen(
        reqwest::Method::PATCH,
        &format!("track/entries/{id}"),
        Some(serde_json::json!({ "note": notiz })),
    )
    .await
}

/// `aktion` ist start, stop oder switch. Ohne Projekt stoppt stop alle
/// laufenden Timer.
pub async fn ausloesen(
    aktion: &str,
    projekt: Option<i64>,
    taetigkeit: Option<i64>,
) -> Result<Timer, Fehler> {
    if !matches!(aktion, "start" | "stop" | "switch") {
        return Err(Fehler::neu("invalid_action", None));
    }

    let mut rumpf = serde_json::Map::new();
    if let Some(p) = projekt {
        rumpf.insert("project".into(), p.into());
    }
    if let Some(t) = taetigkeit {
        rumpf.insert("activity".into(), t.into());
    }

    aufrufen(
        reqwest::Method::POST,
        &format!("track/{aktion}"),
        Some(serde_json::Value::Object(rumpf)),
    )
    .await
}
