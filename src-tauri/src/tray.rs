//! Symbol im Infobereich (Windows) bzw. in der Menüleiste (macOS): Zustand
//! läuft oder steht, Tooltip mit Projekt und Dauer, unter macOS die Dauer
//! als Titel neben dem Symbol. Rechtsklick (Windows) bzw. Klick mit
//! gedrückter Ctrl-Taste (macOS) öffnet das Menü mit den Favoriten,
//! Linksklick das Fenster.

use crate::api::{Auswahl, Timer};
use crate::texte::Texte;
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
// icons/menu). macOS hat diese Spalte nicht und bleibt bei reinem Text mit
// ▶ und ● vor Favoriten und laufenden Timern.
#[cfg(windows)]
fn eintrag(
    app: &AppHandle,
    id: impl Into<tauri::menu::MenuId>,
    text: impl AsRef<str>,
    an: bool,
    bild: &'static [u8],
) -> tauri::Result<tauri::menu::IconMenuItem<Wry>> {
    tauri::menu::IconMenuItem::with_id(
        app,
        id,
        text,
        an,
        Image::from_bytes(bild).ok(),
        None::<&str>,
    )
}

#[cfg(not(windows))]
fn eintrag(
    app: &AppHandle,
    id: impl Into<tauri::menu::MenuId>,
    text: impl AsRef<str>,
    an: bool,
    _bild: &'static [u8],
) -> tauri::Result<tauri::menu::MenuItem<Wry>> {
    tauri::menu::MenuItem::with_id(app, id, text, an, None::<&str>)
}

const VOR_START: &str = if cfg!(windows) { "" } else { "▶ " };
const VOR_LAUFEND: &str = if cfg!(windows) { "" } else { "● " };

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
        if timer.running {
            for e in &timer.entries {
                let text = format!(
                    "{VOR_LAUFEND}{} ({})",
                    auswahl.bezeichnung(e.project_id, e.activity_id),
                    dauer(seit(&e.started_at))
                );
                menu = menu.item(&eintrag(
                    app,
                    format!("laufend:{}", e.id),
                    text,
                    false,
                    include_bytes!("../icons/tray/win-laeuft.png"),
                )?);
            }
            menu = menu
                .item(&eintrag(
                    app,
                    "stop",
                    t.alle_stoppen,
                    true,
                    include_bytes!("../icons/menu/stopp.png"),
                )?)
                .separator();
        }

        // Nur die Favoriten (in ihrer Reihenfolge aus Horaka), nicht
        // zuletzt verwendet: das Menü bleibt so immer gleich.
        let mut gesehen = Vec::new();
        for k in &auswahl.favorites {
            let schluessel = (k.project_id, k.activity_id);
            if gesehen.contains(&schluessel) {
                continue;
            }
            gesehen.push(schluessel);
            let id = format!("start:{}:{}", k.project_id, k.activity_id.unwrap_or(0));
            let text = format!(
                "{VOR_START}{}",
                auswahl.bezeichnung(k.project_id, k.activity_id)
            );
            menu = menu.item(&eintrag(
                app,
                id,
                text,
                true,
                include_bytes!("../icons/menu/start.png"),
            )?);
        }
        if !gesehen.is_empty() {
            menu = menu.separator();
        }
    }

    menu.item(&eintrag(
        app,
        "zeigen",
        t.fenster_oeffnen,
        true,
        include_bytes!("../icons/menu/fenster.png"),
    )?)
    .item(&eintrag(
        app,
        "einstellungen",
        t.einstellungen,
        true,
        include_bytes!("../icons/menu/einstellungen.png"),
    )?)
    .item(&eintrag(
        app,
        "app",
        t.im_browser,
        true,
        include_bytes!("../icons/menu/browser.png"),
    )?)
    .separator()
    .item(&eintrag(
        app,
        "beenden",
        t.beenden,
        true,
        include_bytes!("../icons/menu/beenden.png"),
    )?)
    .build()
}
