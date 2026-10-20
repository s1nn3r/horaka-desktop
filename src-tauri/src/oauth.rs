//! Anmeldung bei Horaka über OAuth 2.1 (der Autorisierungsserver des
//! MCP-Servers, app.horaka.com): dynamische Registrierung als öffentliche
//! App, Zustimmung im Browser, Code mit PKCE (S256) über eine
//! Loopback-Adresse `http://127.0.0.1:<port>/callback` (RFC 8252, kein
//! eigenes URL-Schema nötig), Scope `track`. Danach Erneuern mit Rotation
//! und Widerruf beim Trennen.

use crate::api::{http, sprache, Fehler};
use crate::texte::Texte;
use crate::zugang::{jetzt, Zugang};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// So lange wartet die App auf die Zustimmung im Browser.
const WARTEZEIT: Duration = Duration::from_secs(600);

static ABBRUCH: Mutex<Option<oneshot::Sender<()>>> = Mutex::new(None);

/// Adresse der App (Autorisierungsserver), für Tests überschreibbar mit
/// `HORAKA_APP_URL`.
pub fn app_basis() -> String {
    std::env::var("HORAKA_APP_URL")
        .unwrap_or_else(|_| "https://app.horaka.com".to_string())
        .trim_end_matches('/')
        .to_string()
}

#[derive(Deserialize)]
struct Registrierung {
    client_id: String,
}

#[derive(Deserialize)]
struct TokenAntwort {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    #[serde(default)]
    scope: String,
}

#[derive(Deserialize)]
struct OauthFehler {
    error: String,
    error_description: Option<String>,
}

fn netz(e: reqwest::Error) -> Fehler {
    Fehler::neu("network", Some(e.to_string()))
}

async fn token_holen(client_id: &str, felder: &[(&str, &str)]) -> Result<Zugang, Fehler> {
    let antwort = http()
        .post(format!("{}/oauth/token", app_basis()))
        .header("Accept", "application/json")
        .form(felder)
        .send()
        .await
        .map_err(netz)?;

    if !antwort.status().is_success() {
        let fehler = antwort.json::<OauthFehler>().await.ok();
        return Err(Fehler::neu(
            &format!(
                "oauth_{}",
                fehler.as_ref().map_or("error", |f| f.error.as_str())
            ),
            fehler.and_then(|f| f.error_description),
        ));
    }

    let t = antwort
        .json::<TokenAntwort>()
        .await
        .map_err(|e| Fehler::neu("invalid_response", Some(e.to_string())))?;

    Ok(Zugang::Oauth {
        client_id: client_id.to_string(),
        access: t.access_token,
        refresh: t.refresh_token,
        ablauf: jetzt() + t.expires_in,
        rechte: t.scope,
    })
}

pub async fn erneuern(client_id: &str, refresh: &str) -> Result<Zugang, Fehler> {
    token_holen(
        client_id,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh),
            ("client_id", client_id),
        ],
    )
    .await
}

pub async fn widerrufen(client_id: &str, refresh: &str) {
    let _ = http()
        .post(format!("{}/oauth/revoke", app_basis()))
        .form(&[
            ("token", refresh),
            ("token_type_hint", "refresh_token"),
            ("client_id", client_id),
        ])
        .send()
        .await;
}

/// Bricht eine laufende Anmeldung ab (Knopf „Abbrechen“ im Fenster).
pub fn abbrechen() {
    if let Some(sender) = ABBRUCH.lock().unwrap().take() {
        let _ = sender.send(());
    }
}

/// Die ganze Anmeldung: registrieren, Browser öffnen, auf den Rücksprung
/// warten, Code gegen Token tauschen. `oeffnen` öffnet die Adresse im
/// Standardbrowser.
pub async fn anmelden(oeffnen: impl FnOnce(&str) -> Result<(), Fehler>) -> Result<Zugang, Fehler> {
    let lauscher = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| Fehler::neu("oauth_loopback", Some(e.to_string())))?;
    let port = lauscher
        .local_addr()
        .map_err(|e| Fehler::neu("oauth_loopback", Some(e.to_string())))?
        .port();
    let ruecksprung = format!("http://127.0.0.1:{port}/callback");

    let antwort = http()
        .post(format!("{}/oauth/register", app_basis()))
        .json(&serde_json::json!({
            "client_name": "Horaka Desktop",
            "client_uri": "https://www.horaka.com",
            "redirect_uris": [ruecksprung],
            "grant_types": ["authorization_code", "refresh_token"],
            "response_types": ["code"],
            "token_endpoint_auth_method": "none",
        }))
        .send()
        .await
        .map_err(netz)?;
    if !antwort.status().is_success() {
        return Err(Fehler::neu(
            "oauth_register",
            Some(antwort.status().to_string()),
        ));
    }
    let client = antwort
        .json::<Registrierung>()
        .await
        .map_err(|e| Fehler::neu("invalid_response", Some(e.to_string())))?;

    // PKCE: 64 Zeichen aus zwei UUIDs (nur 0-9 und a-f, also zulässig).
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state = uuid::Uuid::new_v4().simple().to_string();

    let adresse = reqwest::Url::parse_with_params(
        &format!("{}/oauth/authorize", app_basis()),
        &[
            ("response_type", "code"),
            ("client_id", client.client_id.as_str()),
            ("redirect_uri", ruecksprung.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("scope", "track"),
            ("state", state.as_str()),
        ],
    )
    .map_err(|e| Fehler::neu("oauth_url", Some(e.to_string())))?;

    let (sender, empfaenger) = oneshot::channel();
    *ABBRUCH.lock().unwrap() = Some(sender);

    oeffnen(adresse.as_str())?;

    let code = tokio::select! {
        ergebnis = tokio::time::timeout(WARTEZEIT, warten(&lauscher, &state)) => {
            ergebnis.map_err(|_| Fehler::neu("oauth_timeout", None))??
        }
        _ = empfaenger => return Err(Fehler::neu("oauth_cancelled", None)),
    };
    ABBRUCH.lock().unwrap().take();

    token_holen(
        &client.client_id,
        &[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", ruecksprung.as_str()),
            ("client_id", client.client_id.as_str()),
            ("code_verifier", verifier.as_str()),
        ],
    )
    .await
}

/// Wartet auf `GET /callback?code=…&state=…` vom Browser und antwortet mit
/// einer kleinen Seite. Andere Anfragen (etwa favicon.ico) bekommen 404.
async fn warten(lauscher: &TcpListener, state: &str) -> Result<String, Fehler> {
    loop {
        let Ok((mut verbindung, _)) = lauscher.accept().await else {
            continue;
        };

        let mut puffer = vec![0u8; 8192];
        let mut gelesen = 0;
        while gelesen < puffer.len() {
            match verbindung.read(&mut puffer[gelesen..]).await {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    gelesen += n;
                    if puffer[..gelesen].windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
            }
        }
        let anfrage = String::from_utf8_lossy(&puffer[..gelesen]);
        let pfad = anfrage
            .lines()
            .next()
            .and_then(|z| z.split_whitespace().nth(1))
            .unwrap_or("/");

        if !pfad.starts_with("/callback") {
            let _ = verbindung
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await;
            continue;
        }

        let url = reqwest::Url::parse(&format!("http://127.0.0.1{pfad}"))
            .map_err(|e| Fehler::neu("oauth_callback", Some(e.to_string())))?;
        let wert = |name: &str| {
            url.query_pairs()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.into_owned())
        };

        let ergebnis = if wert("state").as_deref() != Some(state) {
            Err(Fehler::neu("oauth_state", None))
        } else if let Some(fehler) = wert("error") {
            Err(Fehler::neu(&format!("oauth_{fehler}"), None))
        } else {
            wert("code").ok_or_else(|| Fehler::neu("oauth_callback", None))
        };

        let _ = verbindung
            .write_all(seite(ergebnis.is_ok()).as_bytes())
            .await;
        let _ = verbindung.shutdown().await;
        return ergebnis;
    }
}

fn seite(ok: bool) -> String {
    let texte = Texte::fuer_system();
    let (titel, text) = if ok {
        (texte.oauth_ok_titel, texte.oauth_ok_text)
    } else {
        (texte.oauth_fehler_titel, texte.oauth_fehler_text)
    };
    let sprache = sprache();
    let html = format!(
        "<!doctype html><html lang=\"{sprache}\"><head><meta charset=\"utf-8\"><title>{titel}</title>\
         <meta name=\"viewport\" content=\"width=device-width\"></head>\
         <body style=\"font-family:system-ui,sans-serif;display:grid;place-items:center;min-height:90vh;margin:0;background:#f6f7f5;color:#17201d\">\
         <main style=\"max-width:28rem;padding:2rem;text-align:center\"><h1 style=\"color:#0f6e63;font-size:1.4rem\">{titel}</h1><p>{text}</p></main></body></html>"
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{html}",
        html.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_wie_rfc_7636() {
        // Beispiel aus RFC 7636, Anhang B.
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(
            b"dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk",
        ));
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[tokio::test]
    async fn callback_liefert_den_code_und_prueft_state() {
        let lauscher = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = lauscher.local_addr().unwrap().port();

        let browser = tokio::spawn(async move {
            let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            s.write_all(b"GET /favicon.ico HTTP/1.1\r\nHost: x\r\n\r\n")
                .await
                .unwrap();
            let mut s = tokio::net::TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap();
            s.write_all(b"GET /callback?code=abc&state=s1&iss=x HTTP/1.1\r\nHost: x\r\n\r\n")
                .await
                .unwrap();
            let mut antwort = String::new();
            s.read_to_string(&mut antwort).await.unwrap();
            antwort
        });

        assert_eq!(warten(&lauscher, "s1").await.unwrap(), "abc");
        assert!(browser.await.unwrap().starts_with("HTTP/1.1 200 OK"));
    }
}
