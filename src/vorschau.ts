import { emit } from '@tauri-apps/api/event';
import { mockIPC } from '@tauri-apps/api/mocks';
import { version } from '../package.json';
import type { Auswahl, Einstellungen, Gestoppt, Stand, Timer } from './typen';

// Vorschau im Browser ohne Rust (`npm run dev`, dann /?vorschau, mit
// ?vorschau=leer ohne Verbindung, ?vorschau=update mit Update-Hinweis): ersetzt die Befehle aus src-tauri durch
// Beispieldaten. Nie im Build (nur import.meta.env.DEV).

const auswahl: Auswahl = {
    projects: [
        { id: 1, name: 'Website Relaunch', client_id: 1, client_name: 'Müller GmbH', color: '#2a73a8', status: 'active', activities: [{ id: 10, name: 'Design' }, { id: 11, name: 'Meeting' }] },
        { id: 2, name: 'Wartung', client_id: 1, client_name: 'Müller GmbH', color: '#0f6e63', status: 'active', activities: [{ id: 11, name: 'Meeting' }] },
        { id: 3, name: 'Kampagne Herbst', client_id: 2, client_name: 'Anker AG', color: '#a8650a', status: 'paused', activities: [] },
        { id: 4, name: 'Buchhaltung', client_id: 3, client_name: 'Intern', color: '#8e44ad', status: 'active', activities: [{ id: 11, name: 'Meeting' }] },
    ],
    favorites: [
        { project_id: 1, activity_id: 10 },
        { project_id: 1, activity_id: 11 },
        { project_id: 2, activity_id: null },
        { project_id: 3, activity_id: null },
        { project_id: 4, activity_id: 11 },
    ],
    recent: [{ project_id: 2, activity_id: null }, { project_id: 1, activity_id: 10 }, { project_id: 4, activity_id: 11 }],
    parallel_timers: true,
};

let timer: Timer = {
    running: true,
    count: 1,
    elapsed_seconds: 2843,
    entry: null,
    entries: [{ id: 99, project_id: 1, activity_id: 10, started_at: new Date(Date.now() - 2843_000).toISOString(), note: null }],
};
let verbunden = new URLSearchParams(location.search).get('vorschau') !== 'leer';
let autostart = true;

const modus = new URLSearchParams(location.search).get('vorschau');
// ?vorschau=offline: verbunden, aber ohne Netz gestartet (Daten fehlen).
let offline = modus === 'offline';
if (offline) {
    window.setTimeout(() => {
        offline = false;
        void emit('stand', stand());
    }, 6000);
}
let abschluss: Gestoppt[] = [];
let einstellungen: Einstellungen = {
    thema: 'system',
    // Über das Neuladen beim Sprachwechsel hinweg merken.
    sprache: sessionStorage.getItem('vorschau-sprache') ?? 'system',
    kompakt: sessionStorage.getItem('vorschau-kompakt') === '1',
    kompakt_durchsichtig: false,
    kompakt_deckkraft: 30,
    notiz_beim_stoppen: true,
    updates_suchen: true,
    kuerzel: [
        { kuerzel: 'control+alt+KeyH', aktion: 'fenster', projekt: null, taetigkeit: null },
        { kuerzel: 'control+alt+Digit1', aktion: 'umschalten', projekt: 1, taetigkeit: 10 },
    ],
};

const stand = (): Stand => ({
    verbunden,
    offline,
    timer: verbunden && !offline ? timer : null,
    auswahl: verbunden && !offline ? auswahl : null,
    abschluss,
    update: modus === 'update' ? version : null,
    update_laeuft: false,
});

mockIPC(
    (befehl, args) => {
        const a = (args ?? {}) as Record<string, unknown>;
        switch (befehl) {
            case 'stand_lesen':
            case 'neu_laden':
                return stand();
            case 'verbinden':
                verbunden = true;
                return stand();
            case 'anmelden':
                // Wie im Browser: kurz warten, dann verbunden.
                return new Promise((fertig) =>
                    window.setTimeout(() => {
                        verbunden = true;
                        fertig(stand());
                    }, 1500),
                );
            case 'trennen':
                verbunden = false;
                return stand();
            case 'ausloesen': {
                const projekt = a.projekt as number | null;
                const taetigkeit = a.taetigkeit as number | null;
                if (a.aktion === 'stop') {
                    const bleiben = projekt === null ? [] : timer.entries.filter((e) => e.project_id !== projekt);
                    if (einstellungen.notiz_beim_stoppen) {
                        abschluss = [
                            ...abschluss,
                            ...timer.entries
                                .filter((e) => !bleiben.includes(e))
                                .map((e) => ({
                                    id: e.id,
                                    project_id: e.project_id,
                                    activity_id: e.activity_id,
                                    note: e.note,
                                    sekunden: Math.floor((Date.now() - Date.parse(e.started_at)) / 1000),
                                })),
                        ];
                    }
                    timer.entries = bleiben;
                } else {
                    if (a.aktion === 'switch' || !auswahl.parallel_timers) {
                        timer.entries = [];
                    }
                    timer.entries.push({ id: Date.now(), project_id: projekt ?? 0, activity_id: taetigkeit, started_at: new Date().toISOString(), note: null });
                }
                timer = { ...timer, running: timer.entries.length > 0, count: timer.entries.length };
                void emit('stand', stand());
                return stand();
            }
            case 'notiz_setzen': {
                const id = a.id as number;
                const notiz = String(a.notiz ?? '').trim() || null;
                timer.entries = timer.entries.map((e) => (e.id === id ? { ...e, note: notiz } : e));
                abschluss = abschluss.filter((g) => g.id !== id);
                return stand();
            }
            case 'notiz_vorschlaege':
                return ['Workshop vorbereitet', 'Folien an Frau Müller verschickt, Rückmeldung bis Freitag', 'Abstimmung Startseite'];
            case 'eintrag_verwerfen':
                timer.entries = timer.entries.filter((e) => e.id !== (a.id as number));
                timer = { ...timer, running: timer.entries.length > 0, count: timer.entries.length };
                return stand();
            // Das native Kontextmenü gibt es im Browser nicht, der Rechtsklick tut dort nichts.
            case 'plugin:menu|new':
                return [1, 'vorschau'];
            case 'abschluss_ueberspringen':
                abschluss = abschluss.filter((g) => g.id !== (a.id as number));
                return stand();
            case 'einstellungen_lesen':
                return einstellungen;
            case 'einstellungen_speichern':
                einstellungen = a.neu as Einstellungen;
                sessionStorage.setItem('vorschau-sprache', einstellungen.sprache);
                sessionStorage.setItem('vorschau-kompakt', einstellungen.kompakt ? '1' : '');
                return { einstellungen, belegt: [] };
            case 'update_suchen':
                return stand();
            case 'update_installieren':
                return null;
            case 'groesse_setzen':
                // Zum Nachsehen in der Konsole, das Browserfenster bleibt, wie es ist.
                (window as unknown as { vorschauGroesse: unknown }).vorschauGroesse = args;
                return null;
            case 'kuerzel_pausieren':
                return null;
            case 'kuerzel_fortsetzen':
                return [];
            case 'plugin:app|version':
                return version;
            case 'plugin:autostart|is_enabled':
                return autostart;
            case 'plugin:autostart|enable':
                autostart = true;
                return null;
            case 'plugin:autostart|disable':
                autostart = false;
                return null;
            default:
                return null;
        }
    },
    { shouldMockEvents: true },
);
