//! Play- und Stopp-Zeichen für das Tray-Menü (Windows) in der Farbe des
//! Projekts: 32 × 32 Pixel RGBA, Formen wie in der App (16er Raster: Play
//! als Dreieck 4.5/2.8, 4.5/13.2, 13/8, Stopp als Quadrat 3.5 bis 12.5 mit
//! Rundung 1.5), Kanten mit 4 × 4 Abtastpunkten je Pixel geglättet.
//!
//! Die API liefert die Farbe als `#rrggbb`.

pub const GROESSE: u32 = 32;

#[derive(Clone, Copy)]
pub enum Form {
    Start,
    Stopp,
}

/// `#rrggbb` als Farbe; alles andere ergibt das Grün der App.
fn farbe(hex: &str) -> [u8; 3] {
    let h = hex.trim_start_matches('#');
    let teil = |i: usize| h.get(i..i + 2).and_then(|t| u8::from_str_radix(t, 16).ok());
    match (h.len(), teil(0), teil(2), teil(4)) {
        (6, Some(r), Some(g), Some(b)) => [r, g, b],
        _ => [0x2a, 0x9d, 0x8f],
    }
}

pub fn rgba(hex: &str, form: Form) -> Vec<u8> {
    let [r, g, b] = farbe(hex);
    let s = GROESSE as f32 / 16.0;
    let (a, b2, c) = ((4.5 * s, 2.8 * s), (4.5 * s, 13.2 * s), (13.0 * s, 8.0 * s));
    // Innen, wenn der Punkt auf derselben Seite aller drei Kanten liegt.
    let kante = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
        (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0)
    };
    let dreieck = |x: f32, y: f32| {
        let (k1, k2, k3) = (kante(a, b2, x, y), kante(b2, c, x, y), kante(c, a, x, y));
        (k1 <= 0.0 && k2 <= 0.0 && k3 <= 0.0) || (k1 >= 0.0 && k2 >= 0.0 && k3 >= 0.0)
    };
    // Quadrat mit runden Ecken: Abstand zum inneren Rechteck höchstens die Rundung.
    let (von, bis, rund) = (3.5 * s, 12.5 * s, 1.5 * s);
    let quadrat = |x: f32, y: f32| {
        let dx = (von + rund - x).max(x - (bis - rund)).max(0.0);
        let dy = (von + rund - y).max(y - (bis - rund)).max(0.0);
        (von..=bis).contains(&x) && (von..=bis).contains(&y) && dx * dx + dy * dy <= rund * rund
    };
    let innen = |x: f32, y: f32| match form {
        Form::Start => dreieck(x, y),
        Form::Stopp => quadrat(x, y),
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
        let bild = rgba("#2563eb", Form::Start);
        assert_eq!(bild.len(), (GROESSE * GROESSE * 4) as usize);
        assert_eq!(pixel(&bild, 14, 16), &[0x25, 0x63, 0xeb, 255]);
        assert_eq!(pixel(&bild, 0, 0)[3], 0);
        assert_eq!(pixel(&bild, 31, 31)[3], 0);
        assert_eq!(pixel(&bild, 28, 16)[3], 0);
    }

    #[test]
    fn falsche_farbe_ergibt_gruen() {
        assert_eq!(&rgba("blau", Form::Start)[..3], &[0x2a, 0x9d, 0x8f]);
        assert_eq!(&rgba("#12345", Form::Start)[..3], &[0x2a, 0x9d, 0x8f]);
    }

    #[test]
    fn stopp_als_quadrat_mit_runden_ecken() {
        let bild = rgba("#c0392b", Form::Stopp);
        assert_eq!(pixel(&bild, 16, 16), &[0xc0, 0x39, 0x2b, 255]);
        assert_eq!(pixel(&bild, 8, 23)[3], 255);
        assert_eq!(pixel(&bild, 7, 7)[3], 0, "Ecke rund");
        assert_eq!(pixel(&bild, 3, 16)[3], 0);
        assert_eq!(pixel(&bild, 26, 16)[3], 0);
    }
}
