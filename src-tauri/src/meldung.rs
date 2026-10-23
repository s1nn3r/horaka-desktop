//! Was eine Aktion bewirkt hat, als Text für die Benachrichtigung des
//! Betriebssystems (Aktionen aus Tastenkürzel und Tray-Menü, wenn das
//! Fenster nicht im Vordergrund ist), und welche Einträge sie gestoppt hat
//! (für die Frage nach der Notiz beim Stoppen). Der Vergleich läuft über die
//! laufenden Einträge vorher und nachher, so passt er zu start, stop,
//! switch und zu parallelen Timern.

use crate::api::{Auswahl, Eintrag, Timer};
use crate::texte::Texte;
use crate::tray::{dauer, seit};
use serde::Serialize;

/// Ein gerade gestoppter Eintrag, zu dem die App nach der Notiz fragt.
#[derive(Debug, Clone, Serialize)]
pub struct Gestoppt {
    pub id: i64,
    pub project_id: i64,
    pub activity_id: Option<i64>,
    pub note: Option<String>,
    pub sekunden: i64,
    pub started_at: String,
    /// Zeitpunkt des Stoppens hier; der Server rundet das Ende noch.
    pub ended_at: String,
}

fn laufende(timer: Option<&Timer>) -> &[Eintrag] {
    timer.map_or(&[], |t| t.entries.as_slice())
}

pub fn gestoppte(vorher: Option<&Timer>, nachher: &Timer) -> Vec<Gestoppt> {
    laufende(vorher)
        .iter()
        .filter(|e| !nachher.entries.iter().any(|n| n.id == e.id))
        .map(|e| Gestoppt {
            id: e.id,
            project_id: e.project_id,
            activity_id: e.activity_id,
            note: e.note.clone(),
            sekunden: seit(&e.started_at),
            started_at: e.started_at.clone(),
            ended_at: chrono::Utc::now().to_rfc3339(),
        })
        .collect()
}

fn name(auswahl: Option<&Auswahl>, e: &Eintrag) -> String {
    auswahl.map_or_else(
        || format!("#{}", e.project_id),
        |a| a.bezeichnung(e.project_id, e.activity_id),
    )
}

/// Titel und Text der Benachrichtigung.
pub fn aenderung(
    t: &Texte,
    vorher: Option<&Timer>,
    nachher: &Timer,
    auswahl: Option<&Auswahl>,
) -> (String, String) {
    let alt = laufende(vorher);
    let gestartet: Vec<&Eintrag> = nachher
        .entries
        .iter()
        .filter(|n| !alt.iter().any(|e| e.id == n.id))
        .collect();
    let gestoppt: Vec<&Eintrag> = alt
        .iter()
        .filter(|e| !nachher.entries.iter().any(|n| n.id == e.id))
        .collect();

    let mit_dauer = |e: &Eintrag| format!("{} · {}", name(auswahl, e), dauer(seit(&e.started_at)));

    match (gestartet.is_empty(), gestoppt.is_empty()) {
        (false, false) => {
            let mut zeilen: Vec<String> = gestartet
                .iter()
                .map(|e| format!("▶ {}", name(auswahl, e)))
                .collect();
            zeilen.extend(gestoppt.iter().map(|e| format!("■ {}", mit_dauer(e))));
            (t.gewechselt.into(), zeilen.join("\n"))
        }
        (false, true) => (
            t.gestartet.into(),
            gestartet
                .iter()
                .map(|e| name(auswahl, e))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        (true, false) => (
            t.gestoppt.into(),
            gestoppt
                .iter()
                .map(|e| mit_dauer(e))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        (true, true) => match nachher.entries.last() {
            Some(e) => (t.laeuft_bereits.into(), mit_dauer(e)),
            None => (t.gestoppt.into(), t.nichts_lief.into()),
        },
    }
}

/// Text für einen Fehler einer Aktion im Hintergrund.
pub fn fehlertext(t: &Texte, code: &str, detail: Option<&str>) -> String {
    match code {
        "network" => t.fehler_netz.into(),
        "insufficient_scope" => t.fehler_recht.into(),
        _ => detail.unwrap_or(t.fehler_sonst).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eintrag(id: i64, projekt: i64) -> Eintrag {
        Eintrag {
            id,
            project_id: projekt,
            activity_id: None,
            started_at: (chrono::Utc::now() - chrono::Duration::minutes(83)).to_rfc3339(),
            note: Some("Notiz".into()),
        }
    }

    fn timer(eintraege: Vec<Eintrag>) -> Timer {
        Timer {
            running: !eintraege.is_empty(),
            count: eintraege.len() as i64,
            elapsed_seconds: None,
            entry: eintraege.last().cloned(),
            entries: eintraege,
        }
    }

    #[test]
    fn erkennt_start_stopp_und_wechsel() {
        let t = Texte::de();
        let leer = timer(vec![]);
        let a = timer(vec![eintrag(1, 10)]);
        let b = timer(vec![eintrag(2, 20)]);

        assert_eq!(aenderung(t, Some(&leer), &a, None).0, "Timer gestartet");
        let (titel, text) = aenderung(t, Some(&a), &leer, None);
        assert_eq!(titel, "Timer gestoppt");
        assert_eq!(text, "#10 · 1:23");
        assert_eq!(aenderung(t, Some(&a), &b, None).0, "Timer gewechselt");
        assert_eq!(aenderung(t, Some(&a), &a, None).0, "Läuft bereits");
        assert_eq!(
            aenderung(t, Some(&leer), &leer, None).1,
            "Es lief kein Timer."
        );
    }

    #[test]
    fn gestoppte_mit_notiz_und_dauer() {
        let vorher = timer(vec![eintrag(1, 10), eintrag(2, 20)]);
        let nachher = timer(vec![eintrag(2, 20)]);
        let g = gestoppte(Some(&vorher), &nachher);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].id, 1);
        assert_eq!(g[0].note.as_deref(), Some("Notiz"));
        assert!((4979..=4981).contains(&g[0].sekunden));
        assert_eq!(g[0].started_at, vorher.entries[0].started_at);
    }
}
