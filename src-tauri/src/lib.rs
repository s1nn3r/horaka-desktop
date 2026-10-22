//! Horaka Desktop: Timer von Horaka im Infobereich (Windows) und in der
//! Menüleiste (macOS). Spricht nur über die REST-API (`api.rs`), hält den
//! letzten Stand im Speicher, gleicht alle 30 Sekunden ab (ohne Netz alle 5
//! Sekunden, bis es wieder geht) und schickt jeden neuen Stand als Ereignis
//! `stand` an das Fenster.
//!
//! Das Fenster öffnet am Symbol und schließt beim Wegklicken wie ein Menü.
//! Zieht man es an der Kopfzeile woanders hin, bleibt es dort (auch nach
//! einem Neustart, `fenster.json`) und schließt nicht mehr von selbst. In der Kompaktansicht
//! ist das Fenster klein, passt seine Höhe dem Inhalt an (`groesse_setzen`),
//! schließt nie von selbst und öffnet beim Start.
//!
//! Globale Tastenkürzel (frei belegbar, `einstellungen.rs`) und das
//! Tray-Menü lösen Aktionen im Hintergrund aus; was passiert ist, meldet eine
//! Benachrichtigung des Betriebssystems (`meldung.rs`). Gestoppte Einträge
//! landen im Abschluss: das Fenster fragt nach der Notiz (abschaltbar).

mod aktualisieren;
mod api;
mod einstellungen;
mod meldung;
mod oauth;
mod texte;
mod tray;
mod zeichen;
mod zeit;
mod zugang;

use api::{Auswahl, Fehler, Timer};
use einstellungen::{Aktion, Einstellungen};
use meldung::Gestoppt;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow, WindowEvent,
};
use tauri_plugin_global_shortcut::{Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_positioner::{Position, WindowExt};

const APP_URL: &str = "https://app.horaka.com";
const FENSTER: &str = "main";

#[derive(Default)]
struct Zustand {
    timer: Mutex<Option<Timer>>,
    auswahl: Mutex<Option<Auswahl>>,
    /// Letzter Abgleich scheiterte am Netz.
    offline: Mutex<bool>,
    verborgen_um: Mutex<Option<Instant>>,
    /// Vom Nutzer gewählte Position des Fensters (physische Pixel).
    ort: Mutex<Option<(i32, i32)>>,
    /// Wann die App das Fenster selbst bewegt hat (das ist kein Ziehen).
    eigene_bewegung: Mutex<Option<Instant>>,
    einstellungen: Mutex<Einstellungen>,
    /// Gestoppte Einträge, zu denen das Fenster nach der Notiz fragt.
    abschluss: Mutex<Vec<Gestoppt>>,
    /// Wann zuletzt an der Kopfzeile gezogen wurde (Druck oder Bewegung).
    ziehen_um: Mutex<Option<Instant>>,
    /// Das Fenster war zu, als der Abschluss es öffnete: ist die letzte
    /// Notiz erledigt, verschwindet es wieder.
    abschluss_wieder_zu: Mutex<bool>,
    /// Zuletzt gesehene Bildschirme (Lage und Größe), um An- und Abstecken
    /// zu erkennen.
    bildschirme: Mutex<Vec<Bildschirm>>,
}

type Bildschirm = (i32, i32, u32, u32);

/// Was das Fenster anzeigt. `verbunden` heißt: ein Zugang ist gespeichert,
/// auch wenn die Daten ohne Netz noch fehlen.
#[derive(Clone, serde::Serialize)]
struct Stand {
    verbunden: bool,
    offline: bool,
    timer: Option<Timer>,
    auswahl: Option<Auswahl>,
    abschluss: Vec<Gestoppt>,
    /// Version eines gefundenen Updates.
    update: Option<String>,
    /// Update wird gerade geladen und installiert.
    update_laeuft: bool,
}

fn stand(app: &AppHandle) -> Stand {
    let z = app.state::<Zustand>();
    let auswahl = z.auswahl.lock().unwrap().clone();
    let timer = z.timer.lock().unwrap().clone();
    let offline = *z.offline.lock().unwrap();
    let abschluss = z.abschluss.lock().unwrap().clone();
    Stand {
        verbunden: zugang::vorhanden(),
        offline,
        timer,
        auswahl,
        abschluss,
        update: aktualisieren::version(app),
        update_laeuft: aktualisieren::laeuft(app),
    }
}

/// Netz da oder weg; ändert sich das, erfährt es das Fenster.
fn offline_setzen(app: &AppHandle, offline: bool) {
    let geaendert = {
        let z = app.state::<Zustand>();
        let mut alt = z.offline.lock().unwrap();
        let geaendert = *alt != offline;
        *alt = offline;
        geaendert
    };
    if geaendert {
        let _ = app.emit("stand", stand(app));
    }
}

fn tray_neu(app: &AppHandle) {
    let z = app.state::<Zustand>();
    let timer = z.timer.lock().unwrap().clone();
    let auswahl = z.auswahl.lock().unwrap().clone();
    tray::aktualisieren(app, timer.as_ref(), auswahl.as_ref());
}

fn uebernehmen(app: &AppHandle, timer: Timer) {
    *app.state::<Zustand>().timer.lock().unwrap() = Some(timer);
    tray_neu(app);
    let _ = app.emit("stand", stand(app));
}

/// Auswahl (wenn verlangt oder noch keine da) und Timer neu holen.
async fn abgleichen(app: &AppHandle, auswahl_neu: bool) -> Result<(), Fehler> {
    let fehlt = app.state::<Zustand>().auswahl.lock().unwrap().is_none();
    if auswahl_neu || fehlt {
        let auswahl = api::auswahl().await?;
        *app.state::<Zustand>().auswahl.lock().unwrap() = Some(auswahl);
    }
    let timer = api::timer().await?;
    *app.state::<Zustand>().offline.lock().unwrap() = false;
    uebernehmen(app, timer);
    Ok(())
}

/// Ein abgelaufener oder widerrufener Token trennt die Verbindung; ohne Netz
/// gilt die App als offline, bis der nächste Abgleich klappt.
fn fehler_behandeln(app: &AppHandle, fehler: Fehler) -> Fehler {
    if fehler.code == "network" {
        offline_setzen(app, true);
    } else if fehler.code == "unauthorized" {
        zugang::loeschen();
        let z = app.state::<Zustand>();
        *z.timer.lock().unwrap() = None;
        *z.auswahl.lock().unwrap() = None;
        tray_neu(app);
        let _ = app.emit("stand", stand(app));
    }
    fehler
}

/// Benachrichtigung des Betriebssystems, außer das Fenster ist gerade vorn
/// (dann sieht man es dort).
fn benachrichtigen(app: &AppHandle, titel: &str, text: &str) {
    let vorn = app
        .get_webview_window(FENSTER)
        .is_some_and(|w| w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false));
    if !vorn {
        let _ = app.notification().builder().title(titel).body(text).show();
    }
}

/// Gestoppte Einträge in den Abschluss legen, wenn nach der Notiz gefragt
/// werden soll; aus dem Hintergrund öffnet sich dazu das Fenster.
fn abschluss_merken(app: &AppHandle, vorher: Option<&Timer>, nachher: &Timer, zeigen: bool) {
    let fragen = app
        .state::<Zustand>()
        .einstellungen
        .lock()
        .unwrap()
        .notiz_beim_stoppen;
    let gestoppt = meldung::gestoppte(vorher, nachher);
    if !fragen || gestoppt.is_empty() {
        return;
    }
    app.state::<Zustand>()
        .abschluss
        .lock()
        .unwrap()
        .extend(gestoppt);
    let _ = app.emit("stand", stand(app));
    if zeigen {
        let offen = app
            .get_webview_window(FENSTER)
            .is_some_and(|w| w.is_visible().unwrap_or(false));
        if !offen {
            *app.state::<Zustand>().abschluss_wieder_zu.lock().unwrap() = true;
        }
        fenster_zeigen(app);
    }
}

/// Ein Eintrag des Abschlusses ist erledigt (Notiz gespeichert oder
/// übersprungen). War das der letzte und hatte erst der Abschluss das
/// Fenster geöffnet, geht es wieder zu.
fn abschluss_erledigt(app: &AppHandle, id: i64) {
    let z = app.state::<Zustand>();
    let leer = {
        let mut liste = z.abschluss.lock().unwrap();
        liste.retain(|g| g.id != id);
        liste.is_empty()
    };
    let wieder_zu = leer && std::mem::take(&mut *z.abschluss_wieder_zu.lock().unwrap());
    if wieder_zu {
        if let Some(w) = app.get_webview_window(FENSTER) {
            let _ = w.hide();
        }
    }
}

/// Eine Aktion aus Tastenkürzel oder Tray-Menü: ausführen, melden, bei
/// gestoppten Einträgen nach der Notiz fragen.
fn aktion_im_hintergrund(
    app: &AppHandle,
    aktion: &'static str,
    projekt: Option<i64>,
    taetigkeit: Option<i64>,
) {
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        let t = texte::Texte::fuer_system();
        let vorher = h.state::<Zustand>().timer.lock().unwrap().clone();
        match api::ausloesen(aktion, projekt, taetigkeit).await {
            Ok(timer) => {
                let auswahl = h.state::<Zustand>().auswahl.lock().unwrap().clone();
                let (titel, text) =
                    meldung::aenderung(t, vorher.as_ref(), &timer, auswahl.as_ref());
                benachrichtigen(&h, &titel, &text);
                uebernehmen(&h, timer.clone());
                abschluss_merken(&h, vorher.as_ref(), &timer, true);
                let _ = abgleichen(&h, true).await;
            }
            Err(e) => {
                let e = fehler_behandeln(&h, e);
                benachrichtigen(
                    &h,
                    t.fehler,
                    &meldung::fehlertext(t, &e.code, e.detail.as_deref()),
                );
                let _ = h.emit("fehler", e);
            }
        }
    });
}

/// Ein globales Tastenkürzel wurde gedrückt.
fn kuerzel_ausfuehren(app: &AppHandle, gedrueckt: &Shortcut) {
    let Some(k) = app
        .state::<Zustand>()
        .einstellungen
        .lock()
        .unwrap()
        .fuer(gedrueckt)
    else {
        return;
    };
    match k.aktion {
        Aktion::Fenster => fenster_umschalten(app),
        Aktion::Notiz => {
            fenster_zeigen(app);
            let _ = app.emit("notiz_fokus", ());
        }
        Aktion::Stoppen => aktion_im_hintergrund(app, "stop", None, None),
        Aktion::Wechseln => aktion_im_hintergrund(app, "switch", k.projekt, k.taetigkeit),
        Aktion::Umschalten => {
            // Läuft die Kombination (ohne Tätigkeit: das Projekt), stoppt das
            // Kürzel sie, sonst startet es sie.
            let laeuft = app
                .state::<Zustand>()
                .timer
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|t| {
                    t.entries.iter().any(|e| {
                        Some(e.project_id) == k.projekt
                            && (k.taetigkeit.is_none() || e.activity_id == k.taetigkeit)
                    })
                });
            let aktion = if laeuft { "stop" } else { "start" };
            aktion_im_hintergrund(app, aktion, k.projekt, k.taetigkeit);
        }
    }
}

pub fn menu_ereignis(app: &AppHandle, id: &str) {
    match id {
        "zeigen" => fenster_zeigen(app),
        // Fenster öffnen, das Fenster wechselt in die Einstellungen.
        "einstellungen" => {
            fenster_zeigen(app);
            let _ = app.emit("einstellungen_oeffnen", ());
        }
        "app" => {
            let _ = app.opener().open_url(APP_URL, None::<&str>);
        }
        "beenden" => app.exit(0),
        "stop" => aktion_im_hintergrund(app, "stop", None, None),
        _ => {
            // Laufende Zeile im Menü: dieses Projekt stoppen.
            if let Some(projekt) = id
                .strip_prefix("stopp:")
                .and_then(|s| s.parse::<i64>().ok())
            {
                aktion_im_hintergrund(app, "stop", Some(projekt), None);
                return;
            }
            if let Some(rest) = id.strip_prefix("start:") {
                let mut teile = rest.split(':');
                let projekt = teile.next().and_then(|s| s.parse::<i64>().ok());
                let taetigkeit = teile
                    .next()
                    .and_then(|s| s.parse::<i64>().ok())
                    .filter(|t| *t > 0);
                if projekt.is_some() {
                    aktion_im_hintergrund(app, "start", projekt, taetigkeit);
                }
            }
        }
    }
}

pub fn fenster_zeigen(app: &AppHandle) {
    let Some(w) = app.get_webview_window(FENSTER) else {
        return;
    };
    let z = app.state::<Zustand>();
    *z.eigene_bewegung.lock().unwrap() = Some(Instant::now());
    let ort = *z.ort.lock().unwrap();

    match ort {
        Some((x, y)) => {
            let _ = w.set_position(PhysicalPosition::new(x, y));
        }
        None => {
            if an_symbol_setzen(&w).is_err() {
                let _ = w.center();
            }
        }
    }
    in_sichtbaren_bereich(&w);

    let _ = w.show();
    let _ = w.set_focus();
    let _ = app.emit("gezeigt", ());
}

/// macOS: Menüleiste oben, das Fenster hängt darunter; Windows: Taskleiste
/// unten, das Fenster steht darüber.
fn an_symbol_setzen(w: &WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    let platz = Position::TrayCenter;
    #[cfg(not(target_os = "macos"))]
    let platz = Position::TrayBottomCenter;
    w.move_window(platz)
}

/// Schiebt das Fenster ganz in den nutzbaren Bereich seines Bildschirms
/// (ohne Taskleiste und Menüleiste), etwa nach einem Bildschirmwechsel.
fn in_sichtbaren_bereich(w: &WebviewWindow) {
    let (Ok(pos), Ok(groesse)) = (w.outer_position(), w.outer_size()) else {
        return;
    };
    let monitor = w
        .monitor_from_point(f64::from(pos.x), f64::from(pos.y))
        .ok()
        .flatten()
        .or_else(|| w.current_monitor().ok().flatten())
        .or_else(|| w.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return;
    };

    let bereich = monitor.work_area();
    let breite = i32::try_from(groesse.width).unwrap_or(0);
    let hoehe = i32::try_from(groesse.height).unwrap_or(0);
    let links = bereich.position.x;
    let oben = bereich.position.y;
    let rechts = links + i32::try_from(bereich.size.width).unwrap_or(0) - breite;
    let unten = oben + i32::try_from(bereich.size.height).unwrap_or(0) - hoehe;

    let x = pos.x.clamp(links, rechts.max(links));
    let y = pos.y.clamp(oben, unten.max(oben));
    if (x, y) != (pos.x, pos.y) {
        let _ = w.set_position(PhysicalPosition::new(x, y));
    }
}

fn bildschirme(app: &AppHandle) -> Vec<Bildschirm> {
    app.available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            (
                m.position().x,
                m.position().y,
                m.size().width,
                m.size().height,
            )
        })
        .collect()
}

/// Liegt die Mitte des Fensters an `ort` auf einem der Bildschirme?
fn ort_sichtbar(w: &WebviewWindow, ort: (i32, i32), liste: &[Bildschirm]) -> bool {
    let groesse = w.outer_size().unwrap_or_default();
    let x = i64::from(ort.0) + i64::from(groesse.width) / 2;
    let y = i64::from(ort.1) + i64::from(groesse.height) / 2;
    liste.iter().any(|&(bx, by, bw, bh)| {
        x >= i64::from(bx)
            && y >= i64::from(by)
            && x < i64::from(bx) + i64::from(bw)
            && y < i64::from(by) + i64::from(bh)
    })
}

/// Bildschirm ab- und wieder angesteckt: das System schiebt das Fenster auf
/// einen übrigen Bildschirm. Die gemerkte Position bleibt, und ist ihr
/// Bildschirm wieder da, kehrt das Fenster dorthin zurück.
fn bildschirme_beobachten(app: &AppHandle) {
    *app.state::<Zustand>().bildschirme.lock().unwrap() = bildschirme(app);
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let jetzt = bildschirme(&h);
            let z = h.state::<Zustand>();
            if jetzt.is_empty() || *z.bildschirme.lock().unwrap() == jetzt {
                continue;
            }
            *z.bildschirme.lock().unwrap() = jetzt.clone();
            let ort = *z.ort.lock().unwrap();
            let Some(w) = h.get_webview_window(FENSTER) else {
                continue;
            };
            *z.eigene_bewegung.lock().unwrap() = Some(Instant::now());
            if let Some(ort) = ort.filter(|o| ort_sichtbar(&w, *o, &jetzt)) {
                let _ = w.set_position(PhysicalPosition::new(ort.0, ort.1));
            }
            in_sichtbaren_bereich(&w);
        }
    });
}

fn ort_datei(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("fenster.json"))
}

fn ort_laden(app: &AppHandle) -> Option<(i32, i32)> {
    let text = std::fs::read_to_string(ort_datei(app)?).ok()?;
    let wert: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some((
        i32::try_from(wert["x"].as_i64()?).ok()?,
        i32::try_from(wert["y"].as_i64()?).ok()?,
    ))
}

fn ort_speichern(app: &AppHandle, ort: Option<(i32, i32)>) {
    let Some(datei) = ort_datei(app) else {
        return;
    };
    match ort {
        Some((x, y)) => {
            if let Some(ordner) = datei.parent() {
                let _ = std::fs::create_dir_all(ordner);
            }
            let _ = std::fs::write(&datei, serde_json::json!({ "x": x, "y": y }).to_string());
        }
        None => {
            let _ = std::fs::remove_file(&datei);
        }
    }
}

/// Fokus weg im Modus „am Symbol“: wie ein Menü schließen, aber erst nach
/// einem Moment und nicht, wenn gerade an der Kopfzeile gezogen wird. Das
/// System nimmt dem Fenster beim Ziehen kurz den Fokus; ohne diese Prüfung
/// schloss ein Druck auf die Kopfzeile das Fenster, statt es zu verschieben.
fn wegklicken_pruefen(app: &AppHandle) {
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(350)).await;
        let Some(w) = h.get_webview_window(FENSTER) else {
            return;
        };
        if w.is_focused().unwrap_or(false) || !w.is_visible().unwrap_or(false) {
            return;
        }
        let z = h.state::<Zustand>();
        let zieht = z
            .ziehen_um
            .lock()
            .unwrap()
            .is_some_and(|um| um.elapsed() < Duration::from_millis(1500));
        if zieht {
            // Fokus zurück, damit späteres Wegklicken wieder schließt.
            let _ = w.set_focus();
            return;
        }
        let _ = w.hide();
        *z.verborgen_um.lock().unwrap() = Some(Instant::now());
    });
}

/// Das Fenster meldet einen Druck auf die Kopfzeile (Beginn des Ziehens).
#[tauri::command]
fn ziehen_beginnt(app: AppHandle) {
    *app.state::<Zustand>().ziehen_um.lock().unwrap() = Some(Instant::now());
}

pub fn fenster_umschalten(app: &AppHandle) {
    let Some(w) = app.get_webview_window(FENSTER) else {
        return;
    };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
        return;
    }
    // Der Klick auf das Symbol nimmt dem Fenster zuerst den Fokus und
    // verbirgt es; gleich danach heißt der Klick also „zu“.
    let gerade = app
        .state::<Zustand>()
        .verborgen_um
        .lock()
        .unwrap()
        .is_some_and(|um| um.elapsed() < Duration::from_millis(300));
    if !gerade {
        fenster_zeigen(app);
    }
}

#[tauri::command]
fn stand_lesen(app: AppHandle) -> Stand {
    stand(&app)
}

#[tauri::command]
async fn verbinden(app: AppHandle, token: String) -> Result<Stand, Fehler> {
    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(Fehler::neu("token_missing", None));
    }
    // Erst prüfen, dann speichern: ein falscher Token landet nie im Schlüsselbund.
    let auswahl = api::auswahl_mit(&token).await?;
    zugang::speichern(zugang::Zugang::Token {
        token: token.clone(),
    })?;
    *app.state::<Zustand>().auswahl.lock().unwrap() = Some(auswahl);
    abgleichen(&app, false)
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    Ok(stand(&app))
}

/// Anmeldung im Browser (OAuth). Kehrt zurück, sobald die Zustimmung da
/// ist, abgebrochen wurde oder zehn Minuten vergangen sind.
#[tauri::command]
async fn anmelden(app: AppHandle) -> Result<Stand, Fehler> {
    let neu = oauth::anmelden(|adresse| {
        app.opener()
            .open_url(adresse, None::<&str>)
            .map_err(|e| Fehler::neu("browser", Some(e.to_string())))
    })
    .await?;
    // Ohne „Trigger“ kann die App keinen Timer starten; dann gar nicht erst
    // verbinden, sondern sagen, welches Häkchen fehlt.
    if let zugang::Zugang::Oauth {
        client_id,
        refresh,
        rechte,
        ..
    } = &neu
    {
        if !rechte.split_whitespace().any(|r| r == "track") {
            oauth::widerrufen(client_id, refresh).await;
            return Err(Fehler::neu("oauth_scope_track", None));
        }
    }
    let auswahl = api::auswahl_mit(match &neu {
        zugang::Zugang::Oauth { access, .. } => access,
        zugang::Zugang::Token { token } => token,
    })
    .await?;
    zugang::speichern(neu)?;
    *app.state::<Zustand>().auswahl.lock().unwrap() = Some(auswahl);
    abgleichen(&app, false)
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    fenster_zeigen(&app);
    Ok(stand(&app))
}

#[tauri::command]
fn anmeldung_abbrechen() {
    oauth::abbrechen();
}

#[tauri::command]
fn trennen(app: AppHandle) -> Stand {
    zugang::loeschen();
    let z = app.state::<Zustand>();
    *z.timer.lock().unwrap() = None;
    *z.auswahl.lock().unwrap() = None;
    *z.offline.lock().unwrap() = false;
    z.abschluss.lock().unwrap().clear();
    *z.abschluss_wieder_zu.lock().unwrap() = false;
    tray_neu(&app);
    stand(&app)
}

/// Notiz eines Eintrags setzen (laufend oder gerade gestoppt); ein
/// Eintrag im Abschluss ist damit erledigt.
#[tauri::command]
async fn notiz_setzen(app: AppHandle, id: i64, notiz: String) -> Result<Stand, Fehler> {
    let notiz = Some(notiz.trim().to_string()).filter(|n| !n.is_empty());
    api::notiz_setzen(id, notiz.clone())
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    {
        abschluss_erledigt(&app, id);
        let z = app.state::<Zustand>();
        let mut timer = z.timer.lock().unwrap();
        if let Some(timer) = timer.as_mut() {
            for e in timer.entries.iter_mut().filter(|e| e.id == id) {
                e.note = notiz.clone();
            }
        }
    }
    let _ = app.emit("stand", stand(&app));
    Ok(stand(&app))
}

/// Vorschläge unter dem Notizfeld (Pillen zum Anklicken).
#[tauri::command]
async fn notiz_vorschlaege(
    app: AppHandle,
    projekt: i64,
    taetigkeit: Option<i64>,
    notiz: String,
) -> Result<Vec<String>, Fehler> {
    api::notizvorschlaege(projekt, taetigkeit, &notiz)
        .await
        .map_err(|e| fehler_behandeln(&app, e))
}

/// Laufenden Timer verwerfen (Kontextmenü), ohne Frage nach der Notiz.
#[tauri::command]
async fn eintrag_verwerfen(app: AppHandle, id: i64) -> Result<Stand, Fehler> {
    api::verwerfen(id)
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    abgleichen(&app, false)
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    Ok(stand(&app))
}

#[tauri::command]
fn abschluss_ueberspringen(app: AppHandle, id: i64) -> Stand {
    abschluss_erledigt(&app, id);
    stand(&app)
}

#[derive(serde::Serialize)]
struct EinstellungenAntwort {
    einstellungen: Einstellungen,
    /// Kürzel, die das System nicht annahm (belegt oder ungültig).
    belegt: Vec<String>,
}

/// Jetzt nach einem Update suchen (Knopf in den Einstellungen).
#[tauri::command]
async fn update_suchen(app: AppHandle) -> Result<Stand, Fehler> {
    aktualisieren::suchen(&app).await?;
    let _ = app.emit("stand", stand(&app));
    Ok(stand(&app))
}

/// Gefundenes Update installieren; die App startet danach neu.
#[tauri::command]
async fn update_installieren(app: AppHandle) -> Result<(), Fehler> {
    let h = app.clone();
    let _ = h.emit("stand", stand(&h));
    let ergebnis = aktualisieren::installieren(&app).await;
    let _ = h.emit("stand", stand(&h));
    ergebnis
}

/// Beim Start (nach 15 Sekunden) und alle 6 Stunden suchen, wenn es in
/// den Einstellungen an ist; eine neue Version meldet eine Benachrichtigung.
fn updates_regelmaessig(app: &AppHandle) {
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(15)).await;
        let mut gemeldet: Option<String> = None;
        loop {
            let an = h
                .state::<Zustand>()
                .einstellungen
                .lock()
                .unwrap()
                .updates_suchen;
            if an {
                if let Ok(Some(version)) = aktualisieren::suchen(&h).await {
                    let _ = h.emit("stand", stand(&h));
                    if gemeldet.as_deref() != Some(version.as_str()) {
                        let t = texte::Texte::fuer_system();
                        benachrichtigen(
                            &h,
                            t.update_titel,
                            &t.update_text.replace("{version}", &version),
                        );
                        gemeldet = Some(version);
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(6 * 60 * 60)).await;
        }
    });
}

#[tauri::command]
fn einstellungen_lesen(app: AppHandle) -> Einstellungen {
    app.state::<Zustand>().einstellungen.lock().unwrap().clone()
}

#[tauri::command]
fn einstellungen_speichern(app: AppHandle, neu: Einstellungen) -> EinstellungenAntwort {
    let (einstellungen, belegt) = einstellungen::uebernehmen(&app, neu);
    *app.state::<Zustand>().einstellungen.lock().unwrap() = einstellungen.clone();
    api::sprache_waehlen(&einstellungen.sprache);
    tray_neu(&app);
    EinstellungenAntwort {
        einstellungen,
        belegt,
    }
}

/// Während ein Kürzel aufgenommen wird, lösen die alten nicht aus.
#[tauri::command]
fn kuerzel_pausieren(app: AppHandle) {
    einstellungen::kuerzel_abmelden(&app);
}

#[tauri::command]
fn kuerzel_fortsetzen(app: AppHandle) -> Vec<String> {
    let e = app.state::<Zustand>().einstellungen.lock().unwrap().clone();
    einstellungen::kuerzel_anmelden(&app, &e)
}

#[tauri::command]
async fn neu_laden(app: AppHandle) -> Result<Stand, Fehler> {
    if !zugang::vorhanden() {
        return Ok(stand(&app));
    }
    abgleichen(&app, true)
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    Ok(stand(&app))
}

#[tauri::command]
async fn ausloesen(
    app: AppHandle,
    aktion: String,
    projekt: Option<i64>,
    taetigkeit: Option<i64>,
) -> Result<Stand, Fehler> {
    let vorher = app.state::<Zustand>().timer.lock().unwrap().clone();
    let timer = api::ausloesen(&aktion, projekt, taetigkeit)
        .await
        .map_err(|e| fehler_behandeln(&app, e))?;
    uebernehmen(&app, timer.clone());
    abschluss_merken(&app, vorher.as_ref(), &timer, false);
    // Zuletzt verwendet ändert sich mit jedem Start.
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = abgleichen(&h, true).await;
    });
    Ok(stand(&app))
}

/// Kompaktansicht mit Ausblenden: meldet dem Fenster `maus` (true, wenn der
/// Zeiger über dem Fenster ist). Das System schickt einem Fenster ohne Fokus
/// nicht überall Mausereignisse, deshalb fragt Rust die Position selbst ab.
fn maus_beobachten(app: &AppHandle) {
    let h = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut zuletzt: Option<bool> = None;
        loop {
            let aktiv = {
                let e = h.state::<Zustand>().einstellungen.lock().unwrap().clone();
                e.kompakt && e.kompakt_durchsichtig
            };
            if aktiv {
                if let (Some(w), Ok(zeiger)) = (h.get_webview_window(FENSTER), h.cursor_position())
                {
                    if let (Ok(pos), Ok(groesse), Ok(true)) =
                        (w.outer_position(), w.outer_size(), w.is_visible())
                    {
                        let drueber = zeiger.x >= f64::from(pos.x)
                            && zeiger.y >= f64::from(pos.y)
                            && zeiger.x < f64::from(pos.x) + f64::from(groesse.width)
                            && zeiger.y < f64::from(pos.y) + f64::from(groesse.height);
                        if zuletzt != Some(drueber) {
                            zuletzt = Some(drueber);
                            let _ = h.emit("maus", drueber);
                        }
                    }
                }
            } else {
                zuletzt = None;
            }
            tokio::time::sleep(Duration::from_millis(if aktiv { 120 } else { 1000 })).await;
        }
    });
}

/// Fenstergröße in logischen Pixeln (die Kompaktansicht folgt ihrem Inhalt).
#[tauri::command]
fn groesse_setzen(app: AppHandle, breite: f64, hoehe: f64) {
    let Some(w) = app.get_webview_window(FENSTER) else {
        return;
    };
    *app.state::<Zustand>().eigene_bewegung.lock().unwrap() = Some(Instant::now());
    // Die Kompaktansicht soll über allem bleiben, auch nach einem Wechsel.
    let _ = w.set_always_on_top(true);
    let _ = w.set_size(LogicalSize::new(
        breite.clamp(160.0, 720.0),
        hoehe.clamp(60.0, 1000.0),
    ));
    // Am Symbol (nicht gezogen, nicht kompakt) bleibt das Fenster an der
    // Taskleiste, wenn es niedriger wird (Notizfenster nach Inhalt).
    let z = app.state::<Zustand>();
    let am_symbol = z.ort.lock().unwrap().is_none() && !z.einstellungen.lock().unwrap().kompakt;
    if am_symbol {
        let _ = an_symbol_setzen(&w);
    }
    in_sichtbaren_bereich(&w);
}

#[tauri::command]
fn verbergen(app: AppHandle) {
    if let Some(w) = app.get_webview_window(FENSTER) {
        let _ = w.hide();
    }
}

#[tauri::command]
fn app_oeffnen(app: AppHandle, pfad: Option<String>) {
    let ziel = format!(
        "{}/{}",
        APP_URL,
        pfad.unwrap_or_default().trim_start_matches('/')
    );
    let _ = app.opener().open_url(ziel, None::<&str>);
}

pub fn run() {
    tauri::Builder::default()
        // Ein zweiter Start öffnet nur das Fenster der laufenden Instanz.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            fenster_zeigen(app)
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(aktualisieren::Updates::default())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, kuerzel, ereignis| {
                    if ereignis.state() == ShortcutState::Pressed {
                        kuerzel_ausfuehren(app, kuerzel);
                    }
                })
                .build(),
        )
        .manage(Zustand::default())
        .invoke_handler(tauri::generate_handler![
            stand_lesen,
            verbinden,
            anmelden,
            anmeldung_abbrechen,
            trennen,
            neu_laden,
            ausloesen,
            verbergen,
            app_oeffnen,
            notiz_setzen,
            eintrag_verwerfen,
            notiz_vorschlaege,
            abschluss_ueberspringen,
            einstellungen_lesen,
            einstellungen_speichern,
            kuerzel_pausieren,
            kuerzel_fortsetzen,
            ziehen_beginnt,
            groesse_setzen,
            update_suchen,
            update_installieren
        ])
        .setup(|app| {
            // Nur in der Menüleiste, kein Symbol im Dock.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::einrichten(app.handle())?;

            // Einstellungen laden und die Tastenkürzel anmelden (Standard:
            // Strg+Alt+H, macOS ctrl+option+H, öffnet und schließt das Fenster).
            let geladen = einstellungen::laden(app.handle());
            einstellungen::kuerzel_anmelden(app.handle(), &geladen);
            api::sprache_waehlen(&geladen.sprache);
            let kompakt = geladen.kompakt;
            *app.state::<Zustand>().einstellungen.lock().unwrap() = geladen;

            *app.state::<Zustand>().ort.lock().unwrap() = ort_laden(app.handle());

            if let Some(w) = app.get_webview_window(FENSTER) {
                let h = app.handle().clone();
                w.on_window_event(move |ereignis| match ereignis {
                    // Am Symbol schließt das Fenster beim Wegklicken wie ein
                    // Menü. Nicht beim Verbinden (man ist im Browser) und
                    // nicht, wenn es woandershin gezogen wurde.
                    WindowEvent::Focused(false) => {
                        let z = h.state::<Zustand>();
                        let am_symbol = z.ort.lock().unwrap().is_none();
                        let kompakt = z.einstellungen.lock().unwrap().kompakt;
                        let hat_daten = z.auswahl.lock().unwrap().is_some();
                        let fragt = !z.abschluss.lock().unwrap().is_empty();
                        if am_symbol && !kompakt && hat_daten && !fragt {
                            wegklicken_pruefen(&h);
                        }
                    }
                    // Gezogen (nicht von der App bewegt): Position merken.
                    WindowEvent::Moved(pos) => {
                        let z = h.state::<Zustand>();
                        let selbst = z
                            .eigene_bewegung
                            .lock()
                            .unwrap()
                            .is_some_and(|um| um.elapsed() < Duration::from_millis(600));
                        let sichtbar = h
                            .get_webview_window(FENSTER)
                            .and_then(|w| w.is_visible().ok())
                            .unwrap_or(false);
                        // Haben sich die Bildschirme geändert, schiebt das
                        // System das Fenster, nicht der Nutzer.
                        let jetzt = bildschirme(&h);
                        let umgesteckt = *z.bildschirme.lock().unwrap() != jetzt;
                        if !selbst && sichtbar && !umgesteckt {
                            *z.ziehen_um.lock().unwrap() = Some(Instant::now());
                            *z.ort.lock().unwrap() = Some((pos.x, pos.y));
                            ort_speichern(&h, Some((pos.x, pos.y)));
                        }
                    }
                    _ => {}
                });
            }

            // Ohne Zugang zum Verbinden, in der Kompaktansicht immer.
            if !zugang::vorhanden() || kompakt {
                fenster_zeigen(app.handle());
            }

            updates_regelmaessig(app.handle());
            maus_beobachten(app.handle());
            bildschirme_beobachten(app.handle());

            // Abgleich alle 30 Sekunden, die Auswahl alle 5 Minuten; ohne
            // Netz alle 5 Sekunden, damit es gleich weitergeht, wenn es
            // wieder da ist. Netzfehler zeigt das Fenster als Zustand, nicht
            // als Meldung.
            let h = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut runde: u32 = 0;
                loop {
                    if zugang::vorhanden() {
                        if let Err(e) = abgleichen(&h, runde.is_multiple_of(10)).await {
                            let e = fehler_behandeln(&h, e);
                            if e.code != "network" {
                                let _ = h.emit("fehler", e);
                            }
                        }
                    } else {
                        tray_neu(&h);
                    }
                    runde = runde.wrapping_add(1);
                    let offline = *h.state::<Zustand>().offline.lock().unwrap();
                    tokio::time::sleep(Duration::from_secs(if offline { 5 } else { 30 })).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Horaka Desktop starten");
}
