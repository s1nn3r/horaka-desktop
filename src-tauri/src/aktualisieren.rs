//! Updates über das Updater-Plugin von Tauri: `latest.json` im neuesten
//! GitHub-Release, Artefakte signiert mit dem Updater-Schlüssel (der
//! öffentliche Teil steht in tauri.conf.json, das Plugin prüft jede
//! Signatur). Unter Windows läuft der Installer im passiven Modus und fragt
//! nicht nach der alten Version, unter macOS wird die App ersetzt; danach
//! startet sie neu.
//!
//! Gesucht wird beim Start (nach einer kurzen Pause) und alle 6 Stunden,
//! wenn es in den Einstellungen an ist; installiert wird erst auf Klick.

use crate::api::Fehler;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
pub struct Updates {
    /// Gefundenes, noch nicht installiertes Update.
    pub bereit: Mutex<Option<Update>>,
    /// Gerade beim Herunterladen und Installieren.
    pub laeuft: Mutex<bool>,
}

/// Neue Version, falls es eine gibt.
pub fn version(app: &AppHandle) -> Option<String> {
    app.state::<Updates>()
        .bereit
        .lock()
        .unwrap()
        .as_ref()
        .map(|u| u.version.clone())
}

pub fn laeuft(app: &AppHandle) -> bool {
    *app.state::<Updates>().laeuft.lock().unwrap()
}

fn fehler(e: impl std::fmt::Display) -> Fehler {
    Fehler::neu("update", Some(e.to_string()))
}

/// Nach einem Update suchen; gibt die neue Version zurück.
pub async fn suchen(app: &AppHandle) -> Result<Option<String>, Fehler> {
    let update = app
        .updater()
        .map_err(fehler)?
        .check()
        .await
        .map_err(fehler)?;
    *app.state::<Updates>().bereit.lock().unwrap() = update;
    Ok(version(app))
}

/// Gefundenes Update laden, installieren und die App neu starten.
pub async fn installieren(app: &AppHandle) -> Result<(), Fehler> {
    let Some(update) = app.state::<Updates>().bereit.lock().unwrap().clone() else {
        return Err(Fehler::neu("no_update", None));
    };
    *app.state::<Updates>().laeuft.lock().unwrap() = true;
    let ergebnis = update.download_and_install(|_, _| {}, || {}).await;
    *app.state::<Updates>().laeuft.lock().unwrap() = false;
    ergebnis.map_err(fehler)?;
    app.restart();
}
