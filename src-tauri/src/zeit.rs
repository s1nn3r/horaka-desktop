//! Zeitpunkte der API (ISO 8601 mit Versatz, etwa `2026-10-16T09:00:00+02:00`).

use chrono::{DateTime, Utc};

pub fn sekunden_seit(iso: &str) -> Option<i64> {
    let start = DateTime::parse_from_rfc3339(iso).ok()?;
    Some(
        (Utc::now() - start.with_timezone(&Utc))
            .num_seconds()
            .max(0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn liest_zeitpunkte_mit_versatz() {
        let vor_einer_stunde = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let s = sekunden_seit(&vor_einer_stunde).unwrap();
        assert!((3599..=3601).contains(&s));
        assert!(sekunden_seit("kein Datum").is_none());
    }
}
