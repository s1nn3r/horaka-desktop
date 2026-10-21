//! Play-Pfeil für das Tray-Menü (Windows) in der Farbe des Projekts: 32 × 32
//! Pixel RGBA, Form wie das Play-Zeichen der App (16er Raster, Dreieck
//! 4.5/2.8, 4.5/13.2, 13/8), Kanten mit 4 × 4 Abtastpunkten je Pixel geglättet.
//!
//! Die API liefert die Farbe als Schlüssel der Palette von Horaka
//! (config/horaka.php, project_colors), je Thema hell oder dunkel.

const PALETTE: [(&str, &str, &str); 12] = [
    ("teal", "#0f6e63", "#4fb3a4"),
    ("amber", "#a8650a", "#e0a24a"),
    ("indigo", "#5b5fc7", "#8b8ff0"),
    ("red", "#c0392b", "#e06b5e"),
    ("blue", "#2a73a8", "#5fa9e0"),
    ("purple", "#8e44ad", "#b57bd6"),
    ("green", "#3b7d34", "#7cc576"),
    ("pink", "#b8376f", "#e77fae"),
    ("slate", "#56677a", "#9aaabb"),
    ("orange", "#c2501a", "#f08a55"),
    ("olive", "#6f6c12", "#c4bf5a"),
    ("brown", "#8a5a3c", "#c8946f"),
];

pub const GROESSE: u32 = 32;

/// Schlüssel der Palette oder `#rrggbb` als Farbe; alles andere ergibt das
/// Grün der App.
fn farbe(wert: &str, dunkel: bool) -> [u8; 3] {
    let hex = PALETTE
        .iter()
        .find(|(name, _, _)| *name == wert)
        .map_or(wert, |(_, hell, dunkle)| if dunkel { dunkle } else { hell });
    let h = hex.trim_start_matches('#');
    let teil = |i: usize| h.get(i..i + 2).and_then(|t| u8::from_str_radix(t, 16).ok());
    match (h.len(), teil(0), teil(2), teil(4)) {
        (6, Some(r), Some(g), Some(b)) => [r, g, b],
        _ => [0x2a, 0x9d, 0x8f],
    }
}

pub fn rgba(wert: &str, dunkel: bool) -> Vec<u8> {
    let [r, g, b] = farbe(wert, dunkel);
    let s = GROESSE as f32 / 16.0;
    let (a, b2, c) = ((4.5 * s, 2.8 * s), (4.5 * s, 13.2 * s), (13.0 * s, 8.0 * s));
    // Innen, wenn der Punkt auf derselben Seite aller drei Kanten liegt.
    let kante = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
        (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0)
    };
    let innen = |x: f32, y: f32| {
        let (k1, k2, k3) = (kante(a, b2, x, y), kante(b2, c, x, y), kante(c, a, x, y));
        (k1 <= 0.0 && k2 <= 0.0 && k3 <= 0.0) || (k1 >= 0.0 && k2 >= 0.0 && k3 >= 0.0)
    };
    let mut bild = Vec::with_capacity((GROESSE * GROESSE * 4) as usize);
    for py in 0..GROESSE {
        for px in 0..GROESSE {
            let mut treffer = 0u32;
            for sy in 0..4 {
                for sx in 0..4 {
                    let x = px as f32 + (sx as f32 + 0.5) / 4.0;
                    let y = py as f32 + (sy as f32 + 0.5) / 4.0;
                    treffer += u32::from(innen(x, y));
                }
            }
            bild.extend_from_slice(&[r, g, b, (treffer * 255 / 16) as u8]);
        }
    }
    bild
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(bild: &[u8], x: u32, y: u32) -> &[u8] {
        let i = ((y * GROESSE + x) * 4) as usize;
        &bild[i..i + 4]
    }

    #[test]
    fn mitte_in_farbe_ecken_durchsichtig() {
        let bild = rgba("#2563eb", false);
        assert_eq!(bild.len(), (GROESSE * GROESSE * 4) as usize);
        assert_eq!(pixel(&bild, 14, 16), &[0x25, 0x63, 0xeb, 255]);
        assert_eq!(pixel(&bild, 0, 0)[3], 0);
        assert_eq!(pixel(&bild, 31, 31)[3], 0);
        assert_eq!(pixel(&bild, 28, 16)[3], 0);
    }

    #[test]
    fn falsche_farbe_ergibt_gruen() {
        assert_eq!(&rgba("blau", false)[..3], &[0x2a, 0x9d, 0x8f]);
        assert_eq!(&rgba("#12345", false)[..3], &[0x2a, 0x9d, 0x8f]);
    }

    #[test]
    fn schluessel_der_palette_je_thema() {
        assert_eq!(&rgba("blue", false)[..3], &[0x2a, 0x73, 0xa8]);
        assert_eq!(&rgba("blue", true)[..3], &[0x5f, 0xa9, 0xe0]);
        assert_eq!(&rgba("brown", false)[..3], &[0x8a, 0x5a, 0x3c]);
    }
}
