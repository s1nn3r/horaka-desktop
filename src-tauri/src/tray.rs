//! Symbol im Infobereich (Windows) bzw. in der Menüleiste (macOS): Zustand
//! läuft oder steht, Tooltip mit Projekt und Dauer, unter macOS die Dauer
//! als Titel neben dem Symbol. Rechtsklick (Windows) bzw. Klick mit
//! gedrückter Ctrl-Taste (macOS) öffnet das Menü mit den Favoriten,
//! Linksklick das Fenster.

use crate::api::{Auswahl, Eintrag, Timer};
use crate::texte::Texte;
use crate::zeichen::Form;
use tauri::image::Image;
use tauri::menu::{Menu, MenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Wry};

pub const ID: &str = "horaka";

fn symbol(laeuft: bool) -> Image<'static> {
    #[cfg(target_os = "macos")]
    let bytes: &[u8] = if laeuft {
        include_bytes!("../icons/tray/mac-laeuft.png")
    } else {
        include_bytes!("../icons/tray/mac-steht.png")
    };
    #[cfg(not(target_os = "macos"))]
    let bytes: &[u8] = if laeuft {
        include_bytes!("../icons/tray/win-laeuft.png")
    } else {
        include_bytes!("../icons/tray/win-steht.png")
    };
    Image::from_bytes(bytes).expect("Tray-Symbol")
}

// Windows hält im Menü links eine Spalte für Symbole frei: dort stehen
// die Symbole der App (Quellen assets/menu-*.svg, 32 px nach
// icons/menu), vor Favoriten der Play-Pfeil in der Projektfarbe
// (`pfeil.rs`). macOS hat diese Spalte nicht und bleibt bei reinem Text mit
// ▶ und ● vor Favoriten und laufenden Timern.
/// `&` im Menütext markiert sonst ein Tastenkürzel („B&R“ würde zu „BR“).
fn maskiert(text: impl AsRef<str>) -> String {
    text.as_ref().replace('&', "&&")
}

#[cfg(windows)]
fn eintrag(
    app: &AppHandle,
    id: impl Into<tauri::menu::MenuId>,
    text: impl AsRef<str>,
    an: bool,
    bild: Option<Image<'static>>,
) -> tauri::Result<tauri::menu::IconMenuItem<Wry>> {
    tauri::menu::IconMenuItem::with_id(app, id, maskiert(text), an, bild, None::<&str>)
}

#[cfg(not(windows))]
fn eintrag(
    app: &AppHandle,
    id: impl Into<tauri::menu::MenuId>,
    text: impl AsRef<str>,
    an: bool,
    _bild: Option<Image<'static>>,
) -> tauri::Result<tauri::menu::MenuItem<Wry>> {
    tauri::menu::MenuItem::with_id(app, id, maskiert(text), an, None::<&str>)
}

/// Play- oder Stopp-Zeichen in der Farbe des Projekts.
fn zeichen(auswahl: &Auswahl, projekt: i64, form: Form) -> Option<Image<'static>> {
    let farbe = auswahl.projekt(projekt).map_or("", |p| p.color.as_str());
    Some(Image::new_owned(
        crate::zeichen::rgba(farbe, form),
        crate::zeichen::GROESSE,
        crate::zeichen::GROESSE,
    ))
}

/// Text einer laufenden Zeile mit ihrer Dauer. Unter Windows steht die Dauer
/// nach einem Tabulator rechtsbündig (wie Tastenkürzel im Menü), der Text
/// davor höchstens 40 Zeichen; unter macOS in Klammern dahinter.
fn mit_dauer(vor: &str, text: &str, e: &Eintrag) -> String {
    let zeit = dauer(seit(&e.started_at));
    if cfg!(windows) {
        let text = if text.chars().count() > 40 {
            format!("{}…", text.chars().take(39).collect::<String>())
        } else {
            text.to_string()
        };
        format!("{vor}{text}\t{zeit}")
    } else {
        format!("{vor}{text} ({zeit})")
    }
}

fn png(bytes: &'static [u8]) -> Option<Image<'static>> {
    Image::from_bytes(bytes).ok()
}

const VOR_START: &str = if cfg!(windows) { "" } else { "▶ " };
const VOR_LAUFEND: &str = if cfg!(windows) { "" } else { "■ " };

pub fn einrichten(app: &AppHandle) -> tauri::Result<()> {
    let menu = menu_bauen(app, None, None)?;

    TrayIconBuilder::with_id(ID)
        .icon(symbol(false))
        .icon_as_template(cfg!(target_os = "macos"))
        .tooltip("Horaka")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| crate::menu_ereignis(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::fenster_umschalten(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Dauer als „1:05“ (Stunden:Minuten).
pub fn dauer(sekunden: i64) -> String {
    let minuten = sekunden.max(0) / 60;
    format!("{}:{:02}", minuten / 60, minuten % 60)
}

/// Symbol, Tooltip, Titel und Menü nach dem aktuellen Stand.
pub fn aktualisieren(app: &AppHandle, timer: Option<&Timer>, auswahl: Option<&Auswahl>) {
    let Some(tray) = app.tray_by_id(ID) else {
        return;
    };
    let t = Texte::fuer_system();
    let laeuft = timer.map(|t| t.running).unwrap_or(false);

    let tooltip = match (timer, auswahl) {
        (Some(timer), Some(auswahl)) if timer.running => {
            let zeilen: Vec<String> = timer
                .entries
                .iter()
                .map(|e| {
                    format!(
                        "{} ({})",
                        auswahl.bezeichnung(e.project_id, e.activity_id),
                        dauer(seit(&e.started_at))
                    )
                })
                .collect();
            format!("Horaka\n{}", zeilen.join("\n"))
        }
        _ => format!("Horaka · {}", t.kein_timer),
    };

    let _ = tray.set_icon(Some(symbol(laeuft)));
    #[cfg(target_os = "macos")]
    let _ = tray.set_icon_as_template(true);
    let _ = tray.set_tooltip(Some(tooltip));

    // Unter macOS steht die Dauer des zuletzt gestarteten Timers neben dem Symbol.
    #[cfg(target_os = "macos")]
    {
        let titel = timer
            .filter(|t| t.running)
            .and_then(|t| t.entry.as_ref())
            .map(|e| dauer(seit(&e.started_at)));
        let _ = tray.set_title(titel);
    }

    if let Ok(menu) = menu_bauen(app, timer, auswahl) {
        let _ = tray.set_menu(Some(menu));
    }
}

/// Sekunden seit einem Zeitpunkt der API (ISO 8601 mit Versatz).
pub fn seit(iso: &str) -> i64 {
    crate::zeit::sekunden_seit(iso).unwrap_or(0)
}

fn menu_bauen(
    app: &AppHandle,
    timer: Option<&Timer>,
    auswahl: Option<&Auswahl>,
) -> tauri::Result<Menu<Wry>> {
    let t = Texte::fuer_system();
    let mut menu = MenuBuilder::new(app);

    if let (Some(timer), Some(auswahl)) = (timer, auswahl) {
        // Favoriten in ihrer Reihenfolge aus Horaka, doppelte nur einmal,
        // nicht zuletzt verwendet: das Menü bleibt so immer gleich.
        let mut favoriten: Vec<(i64, Option<i64>)> = Vec::new();
        for k in &auswahl.favorites {
            if !favoriten.contains(&(k.project_id, k.activity_id)) {
                favoriten.push((k.project_id, k.activity_id));
            }
        }
        // Wie in der Kompaktansicht: ein Favorit ohne Tätigkeit läuft, wenn
        // irgendetwas im Projekt läuft.
        let passt = |f: &(i64, Option<i64>), e: &Eintrag| {
            f.0 == e.project_id && (f.1.is_none() || f.1 == e.activity_id)
        };
        let laufende: &[Eintrag] = if timer.running { &timer.entries } else { &[] };

        // Laufende Timer ohne Favorit stehen oben, ebenfalls zum Stoppen.
        for e in laufende
            .iter()
            .filter(|e| !favoriten.iter().any(|f| passt(f, e)))
        {
            menu = menu.item(&eintrag(
                app,
                format!("stopp:{}", e.project_id),
                mit_dauer(
                    VOR_LAUFEND,
                    &auswahl.bezeichnung(e.project_id, e.activity_id),
                    e,
                ),
                true,
                zeichen(auswahl, e.project_id, Form::Stopp),
            )?);
        }
        if !laufende.is_empty() {
            menu = menu
                .item(&eintrag(
                    app,
                    "stop",
                    t.alle_stoppen,
                    true,
                    png(include_bytes!("../icons/menu/stopp.png")),
                )?)
                .separator();
        }

        for f in &favoriten {
            let text = auswahl.bezeichnung(f.0, f.1);
            let item = match laufende.iter().find(|e| passt(f, e)) {
                Some(e) => eintrag(
                    app,
                    format!("stopp:{}", f.0),
                    mit_dauer(VOR_LAUFEND, &text, e),
                    true,
                    zeichen(auswahl, f.0, Form::Stopp),
                )?,
                None => eintrag(
                    app,
                    format!("start:{}:{}", f.0, f.1.unwrap_or(0)),
                    format!("{VOR_START}{text}"),
                    true,
                    zeichen(auswahl, f.0, Form::Start),
                )?,
            };
            menu = menu.item(&item);
        }
        if !favoriten.is_empty() {
            menu = menu.separator();
        }
    }

    menu.item(&eintrag(
        app,
        "zeigen",
        t.fenster_oeffnen,
        true,
        png(include_bytes!("../icons/menu/fenster.png")),
    )?)
    .item(&eintrag(
        app,
        "einstellungen",
        t.einstellungen,
        true,
        png(include_bytes!("../icons/menu/einstellungen.png")),
    )?)
    .item(&eintrag(
        app,
        "app",
        t.im_browser,
        true,
        png(include_bytes!("../icons/menu/browser.png")),
    )?)
    .separator()
    .item(&eintrag(
        app,
        "beenden",
        t.beenden,
        true,
        png(include_bytes!("../icons/menu/beenden.png")),
    )?)
    .build()
}
