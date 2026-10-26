//! Einstellungen der App in `einstellungen.json` im Konfigurationsordner
//! (keine Geheimnisse, die liegen im Schlüsselbund): Thema, Sprache,
//! Kompaktansicht, ob beim Stoppen nach einer Notiz gefragt wird, ob nach
//! Updates gesucht wird, und die globalen Tastenkürzel.
//!
//! Ein Kürzel ist ein Text wie `control+alt+KeyH` (Modifikatoren und der
//! `KeyboardEvent.code` der Taste), den `Shortcut::from_str` versteht.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aktion {
    /// Fenster öffnen und schließen.
    Fenster,
    /// Fenster öffnen, Notiz eines laufenden Timers bearbeiten (bei mehreren mit Auswahl).
    Notiz,
    /// Alle Timer stoppen.
    Stoppen,
    /// Kombination starten, oder stoppen, wenn sie läuft.
    Umschalten,
    /// Zur Kombination wechseln (andere Timer enden).
    Wechseln,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kuerzel {
    pub kuerzel: String,
    pub aktion: Aktion,
    #[serde(default)]
    pub projekt: Option<i64>,
    #[serde(default)]
    pub taetigkeit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Einstellungen {
    /// system, hell oder dunkel.
    pub thema: String,
    /// system (wie das Betriebssystem) oder ein Code aus `api::SPRACHEN`.
    pub sprache: String,
    /// Kleines Fenster mit den laufenden Timern, das beim Wegklicken offen bleibt.
    pub kompakt: bool,
    /// Kompaktansicht blendet halb aus, solange die Maus nicht darüber ist.
    pub kompakt_durchsichtig: bool,
    /// Deckkraft der ausgeblendeten Kompaktansicht in Prozent (10 bis 90).
    pub kompakt_deckkraft: u8,
    pub notiz_beim_stoppen: bool,
    /// Beim Start und alle 6 Stunden nach Updates suchen.
    pub updates_suchen: bool,
    pub kuerzel: Vec<Kuerzel>,
}

impl Default for Einstellungen {
    fn default() -> Self {
        Self {
            thema: "system".into(),
            sprache: "system".into(),
            kompakt: false,
            kompakt_durchsichtig: false,
            kompakt_deckkraft: 30,
            notiz_beim_stoppen: true,
            updates_suchen: true,
            kuerzel: vec![Kuerzel {
                kuerzel: "control+alt+KeyH".into(),
                aktion: Aktion::Fenster,
                projekt: None,
                taetigkeit: None,
            }],
        }
    }
}

impl Einstellungen {
    /// Das Kürzel, zu dem ein gedrücktes Shortcut gehört.
    pub fn fuer(&self, gedrueckt: &Shortcut) -> Option<Kuerzel> {
        self.kuerzel
            .iter()
            .find(|k| {
                k.kuerzel
                    .parse::<Shortcut>()
                    .is_ok_and(|s| s.id() == gedrueckt.id())
            })
            .cloned()
    }

    /// Ungültige Werte auf sichere zurücksetzen.
    fn bereinigt(mut self) -> Self {
        if !matches!(self.thema.as_str(), "system" | "hell" | "dunkel") {
            self.thema = "system".into();
        }
        self.kompakt_deckkraft = self.kompakt_deckkraft.clamp(10, 90);
        if self.sprache != "system" && !crate::api::SPRACHEN.contains(&self.sprache.as_str()) {
            self.sprache = "system".into();
        }
        self.kuerzel.retain(|k| {
            let mit_ziel =
                !matches!(k.aktion, Aktion::Umschalten | Aktion::Wechseln) || k.projekt.is_some();
            mit_ziel && k.kuerzel.parse::<Shortcut>().is_ok()
        });
        self
    }
}

fn datei(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("einstellungen.json"))
}

pub fn laden(app: &AppHandle) -> Einstellungen {
    datei(app)
        .and_then(|d| std::fs::read_to_string(d).ok())
        .and_then(|t| serde_json::from_str::<Einstellungen>(&t).ok())
        .unwrap_or_default()
        .bereinigt()
}

pub fn speichern(app: &AppHandle, einstellungen: &Einstellungen) {
    let Some(datei) = datei(app) else {
        return;
    };
    if let Some(ordner) = datei.parent() {
        let _ = std::fs::create_dir_all(ordner);
    }
    if let Ok(json) = serde_json::to_string_pretty(einstellungen) {
        let _ = std::fs::write(datei, json);
    }
}

/// Meldet alle Kürzel neu beim System an. Zurück kommen die, die nicht
/// gingen (ungültig oder schon von einem anderen Programm belegt).
pub fn kuerzel_anmelden(app: &AppHandle, einstellungen: &Einstellungen) -> Vec<String> {
    let system = app.global_shortcut();
    let _ = system.unregister_all();
    let mut belegt = Vec::new();
    for k in &einstellungen.kuerzel {
        let ok = k
            .kuerzel
            .parse::<Shortcut>()
            .is_ok_and(|s| system.register(s).is_ok());
        if !ok {
            belegt.push(k.kuerzel.clone());
        }
    }
    belegt
}

/// Während im Fenster ein Kürzel aufgenommen wird, sollen die alten nicht
/// auslösen.
pub fn kuerzel_abmelden(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
}

pub fn uebernehmen(app: &AppHandle, neu: Einstellungen) -> (Einstellungen, Vec<String>) {
    let neu = neu.bereinigt();
    speichern(app, &neu);
    let belegt = kuerzel_anmelden(app, &neu);
    (neu, belegt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_hat_das_fensterkuerzel() {
        let e = Einstellungen::default();
        assert_eq!(e.thema, "system");
        assert_eq!(e.sprache, "system");
        assert!(!e.kompakt);
        assert_eq!(e.kompakt_deckkraft, 30);
        assert!(e.notiz_beim_stoppen);
        assert_eq!(e.kuerzel.len(), 1);
        assert!(e.kuerzel[0].kuerzel.parse::<Shortcut>().is_ok());
    }

    #[test]
    fn bereinigt_unbekanntes_und_kuerzel_ohne_ziel() {
        let e: Einstellungen = serde_json::from_str(
            r#"{"thema":"bunt","sprache":"klingonisch","kompakt_deckkraft":3,"kuerzel":[
                {"kuerzel":"control+alt+KeyS","aktion":"stoppen"},
                {"kuerzel":"control+alt+Digit1","aktion":"umschalten"},
                {"kuerzel":"control+alt+Digit2","aktion":"wechseln","projekt":7,"taetigkeit":3},
                {"kuerzel":"quatsch","aktion":"fenster"}
            ]}"#,
        )
        .unwrap();
        let e = e.bereinigt();
        assert_eq!(e.thema, "system");
        assert_eq!(e.sprache, "system");
        assert_eq!(e.kompakt_deckkraft, 10);
        assert!(e.notiz_beim_stoppen);
        let namen: Vec<_> = e.kuerzel.iter().map(|k| k.kuerzel.as_str()).collect();
        assert_eq!(namen, ["control+alt+KeyS", "control+alt+Digit2"]);
        let gedrueckt: Shortcut = "ctrl+alt+Digit2".parse().unwrap();
        assert_eq!(e.fuer(&gedrueckt).unwrap().projekt, Some(7));
    }
}
