// Formen wie in src-tauri/src/api.rs (Antworten von api.horaka.com/v1).

export interface Eintrag {
    id: number;
    project_id: number;
    activity_id: number | null;
    started_at: string;
    note: string | null;
}

export interface Timer {
    running: boolean;
    count: number;
    elapsed_seconds: number | null;
    entry: Eintrag | null;
    entries: Eintrag[];
}

export interface Taetigkeit {
    id: number;
    name: string;
}

export interface Projekt {
    id: number;
    name: string;
    client_id: number;
    client_name: string;
    color: string;
    status: 'active' | 'paused';
    activities: Taetigkeit[];
}

export interface Kombination {
    project_id: number;
    activity_id: number | null;
}

export interface Auswahl {
    projects: Projekt[];
    favorites: Kombination[];
    recent: Kombination[];
    parallel_timers: boolean;
}

export interface Stand {
    /** Ein Zugang ist gespeichert (auch wenn ohne Netz noch Daten fehlen). */
    verbunden: boolean;
    /** Der letzte Abgleich scheiterte am Netz; die App versucht es weiter. */
    offline: boolean;
    timer: Timer | null;
    auswahl: Auswahl | null;
    /** Gerade gestoppte Einträge, zu denen die App nach der Notiz fragt. */
    abschluss: Gestoppt[];
    /** Version eines gefundenen Updates. */
    update: string | null;
    /** Update wird gerade geladen und installiert. */
    update_laeuft: boolean;
}

export interface Gestoppt {
    id: number;
    project_id: number;
    activity_id: number | null;
    note: string | null;
    sekunden: number;
}

export type KuerzelAktion = 'fenster' | 'notiz' | 'stoppen' | 'umschalten' | 'wechseln';

export interface Kuerzel {
    /** Etwa `control+alt+KeyH` (Modifikatoren und KeyboardEvent.code). */
    kuerzel: string;
    aktion: KuerzelAktion;
    projekt: number | null;
    taetigkeit: number | null;
}

export type Thema = 'system' | 'hell' | 'dunkel';

export interface Einstellungen {
    thema: Thema;
    /** `system` oder ein Sprachcode (de, en, fr, es, it, nl, pl, pt). */
    sprache: string;
    /** Kleines Fenster mit den laufenden Timern, das offen bleibt. */
    kompakt: boolean;
    /** Kompaktansicht blendet halb aus, solange die Maus nicht darüber ist. */
    kompakt_durchsichtig: boolean;
    /** Deckkraft der ausgeblendeten Kompaktansicht in Prozent (10 bis 90). */
    kompakt_deckkraft: number;
    notiz_beim_stoppen: boolean;
    updates_suchen: boolean;
    kuerzel: Kuerzel[];
}

export interface EinstellungenAntwort {
    einstellungen: Einstellungen;
    /** Kürzel, die das System nicht annahm. */
    belegt: string[];
}

export interface Fehler {
    code: string;
    detail: string | null;
}

export type Aktion = 'start' | 'stop' | 'switch';
