import { es } from './sprachen/es';
import { fr } from './sprachen/fr';
import { it } from './sprachen/it';
import { nl } from './sprachen/nl';
import { pl } from './sprachen/pl';
import { pt } from './sprachen/pt';

// Texte des Fensters in Deutsch (geduzt) und britischem Englisch, weitere
// Sprachen unter src/sprachen, Sprache nach System. Begriffe wie in Horaka:
// client, project, activity.

const de = {
    verbinden: {
        titel: 'Mit Horaka verbinden',
        text: 'Melde dich im Browser bei Horaka an und erlaube Horaka Desktop, deinen Timer zu steuern.',
        anmelden: 'Im Browser anmelden',
        warten: 'Warte auf die Anmeldung im Browser…',
        wartenText: 'Erlaube den Zugriff im Browser. Danach geht es hier von selbst weiter.',
        abbrechen: 'Abbrechen',
        rechteTitel: 'Welche Rechte?',
        rechte: 'Setz auf der Seite nur das Häkchen bei „Trigger“. Damit startet, wechselt und stoppt Horaka Desktop deine Timer. Lesen, Schreiben, Abrechnung und Webhooks braucht die App nicht.',
        alternativ: 'Stattdessen einen Gerätetoken einfügen',
        tokenText: 'Leg in Horaka unter Einstellungen, API und Geräte ein Gerät der Art „Horaka Desktop“ an und füge den Token hier ein.',
        einstellungen: 'Einstellungen in Horaka öffnen',
        token: 'Gerätetoken',
        platzhalter: '123|abc…',
        knopf: 'Verbinden',
        laeuft: 'Verbinde…',
        hinweis: 'Der Zugang liegt im Schlüsselbund deines Systems, nie in einer Datei. Horaka Desktop braucht den Tarif Premium.',
    },
    laden: {
        titel: 'Verbinde mit Horaka…',
        offline: 'Keine Verbindung zu Horaka. Sobald dein Netz wieder da ist, geht es von selbst weiter.',
        erneut: 'Jetzt erneut versuchen',
    },
    haupt: {
        suche: 'Projekt, Kunde oder Tätigkeit suchen',
        laufend: 'Läuft',
        favoriten: 'Favoriten',
        zuletzt: 'Zuletzt verwendet',
        projekte: 'Alle Projekte',
        keine: 'Keine Projekte gefunden.',
        leer: 'Noch keine buchbaren Projekte. Leg in Horaka einen Kunden und ein Projekt an.',
        starten: 'Starten',
        zusaetzlich: 'Zusätzlich starten',
        wechseln: 'Hierhin wechseln',
        stoppen: 'Stoppen',
        alleStoppen: 'Alle stoppen',
        taetigkeiten: 'Tätigkeiten zeigen',
        ohneTaetigkeit: 'ohne Tätigkeit',
        pausiert: 'pausiert',
        oeffnen: 'Horaka im Browser öffnen',
        einstellungen: 'Einstellungen',
        schliessen: 'Schließen',
        notiz: 'Notiz',
        notizPlatzhalter: 'Notiz hinzufügen…',
        notizHinweis: 'Enter speichert, Umschalt+Enter neue Zeile, Esc bricht ab.',
        speichern: 'Speichern',
        abbrechen: 'Abbrechen',
        kompakt: 'Kompaktansicht',
        breite: 'Breite ändern (Pfeiltasten)',
        gross: 'Volle Ansicht',
    },
    abschluss: {
        titel: 'Was hast du gemacht?',
        erfasst: ':dauer erfasst',
        von: ':nr von :anzahl',
        platzhalter: 'Kurz zusammengefasst, landet als Notiz am Eintrag',
        speichern: 'Speichern',
        ohne: 'Ohne Notiz weiter',
        nichtMehr: 'Nicht mehr fragen',
        hinweis: 'Enter speichert, Umschalt+Enter neue Zeile.',
    },
    einstellungen: {
        titel: 'Einstellungen',
        zurueck: 'Zurück',
        autostart: 'Beim Anmelden starten',
        autostartText: 'Horaka Desktop startet mit deinem Rechner und wartet im Hintergrund.',
        trennen: 'Verbindung trennen',
        trennenText: 'Meldet Horaka Desktop ab und löscht den Zugang aus dem Schlüsselbund. Verbundene Apps und Geräte siehst du in Horaka unter API und Geräte.',
        version: 'Version',
        thema: 'Darstellung',
        themen: { system: 'Wie das System', hell: 'Hell', dunkel: 'Dunkel' },
        notizen: 'Notizen',
        notizBeimStoppen: 'Beim Stoppen nach einer Notiz fragen',
        notizBeimStoppenText: 'Nach jedem Stopp, auch per Tastenkürzel oder Menü, öffnet sich das Fenster und fragt, was du gemacht hast.',
        kuerzelTitel: 'Tastenkürzel',
        kuerzelText: 'Funktionieren überall, auch wenn Horaka Desktop im Hintergrund ist. Was passiert ist, meldet eine Benachrichtigung.',
        kuerzelNeu: 'Kürzel hinzufügen',
        kuerzelAktion: 'Was soll passieren?',
        kuerzelZiel: 'Projekt',
        kuerzelTaste: 'Tastenkombination',
        kuerzelAufnehmen: 'Klicken und Tasten drücken',
        kuerzelDruecken: 'Drück jetzt die Tasten…',
        kuerzelRegel: 'Mit Strg, Alt oder Win (Mac: ctrl, option oder cmd), oder eine Taste F1 bis F24.',
        kuerzelHinzufuegen: 'Hinzufügen',
        kuerzelEntfernen: 'Entfernen',
        kuerzelDoppelt: 'Dieses Kürzel ist schon vergeben.',
        kuerzelBelegt: 'Diese Kürzel nimmt das System nicht an, vermutlich nutzt sie ein anderes Programm: :liste',
        kuerzelLeer: 'Noch keine Kürzel.',
        aktionen: {
            fenster: 'Fenster öffnen und schließen',
            notiz: 'Notiz zum laufenden Timer',
            stoppen: 'Alle Timer stoppen',
            umschalten: 'Projekt starten oder stoppen',
            wechseln: 'Zu Projekt wechseln',
        },
        projektWaehlen: 'Projekt wählen…',
        sprache: 'Sprache',
        spracheSystem: 'Wie das System',
        spracheText: 'Das Fenster lädt danach neu.',
        kompakt: 'Kompaktansicht',
        kompaktText: 'Kleines Fenster mit den laufenden Timern, das offen bleibt, auch beim Wegklicken und nach einem Neustart.',
        durchsichtig: 'Ausblenden, wenn die Maus nicht darüber ist',
        durchsichtigText: 'Das kleine Fenster wird durchsichtig und erscheint wieder, sobald du mit der Maus darüberfährst.',
        allgemein: 'Allgemein',
        kuerzelReiter: 'Kürzel',
        system: 'System',
        deckkraft: 'Deckkraft beim Ausblenden',
    },
    update: {
        da: 'Horaka Desktop :version ist da.',
        installieren: 'Jetzt aktualisieren',
        laeuft: 'Wird aktualisiert, die App startet gleich neu…',
        titel: 'Updates',
        suchen: 'Beim Start nach Updates suchen',
        suchenText: 'Auch alle 6 Stunden. Eine neue Version meldet eine Benachrichtigung; installiert wird erst, wenn du es willst.',
        jetzt: 'Jetzt nach Updates suchen',
        sucht: 'Suche…',
        aktuell: 'Du hast die neueste Version.',
    },
    fehler: {
        unauthorized: 'Der Token gilt nicht mehr. Verbinde Horaka Desktop neu.',
        insufficient_scope: 'Horaka Desktop fehlt das Recht „Trigger“. Trenne die Verbindung in den Einstellungen und verbinde neu, mit Häkchen bei „Trigger“.',
        oauth_scope_track: 'Ohne das Recht „Trigger“ kann Horaka Desktop keine Timer steuern. Verbinde neu und setz das Häkchen bei „Trigger“.',
        network: 'Keine Verbindung zu Horaka. Prüfe dein Netz.',
        rate_limited: 'Zu viele Anfragen. Versuch es gleich noch einmal.',
        keychain: 'Der Schlüsselbund lässt sich nicht verwenden.',
        token_missing: 'Füge zuerst einen Token ein.',
        browser: 'Der Browser ließ sich nicht öffnen.',
        oauth_timeout: 'Die Anmeldung hat zu lange gedauert. Versuch es noch einmal.',
        oauth_access_denied: 'Du hast den Zugriff abgelehnt.',
        oauth_state: 'Die Antwort des Browsers passte nicht zur Anmeldung. Versuch es noch einmal.',
        plan_required: 'Dein Tarif erlaubt das nicht. Horaka Desktop braucht Premium.',
        unknown: 'Etwas ist schiefgegangen.',
        update: 'Das Update hat nicht geklappt. Versuch es später noch einmal oder lade die neue Version von der Webseite.',
    },
    tasten: {
        strg: 'Strg',
        umschalt: 'Umschalt',
        leertaste: 'Leertaste',
    },
};

export type Texte = typeof de;

const en: Texte = {
    verbinden: {
        titel: 'Connect to Horaka',
        text: 'Sign in to Horaka in your browser and allow Horaka Desktop to control your timer.',
        anmelden: 'Sign in with the browser',
        warten: 'Waiting for you to sign in…',
        wartenText: 'Allow access in the browser. Horaka Desktop continues on its own afterwards.',
        abbrechen: 'Cancel',
        rechteTitel: 'Which permissions?',
        rechte: 'Tick only “Trigger” on the page. That lets Horaka Desktop start, switch and stop your timers. The app does not need read, write, billing or webhooks.',
        alternativ: 'Paste a device token instead',
        tokenText: 'In Horaka, go to Settings, API and devices, add a device of the kind “Horaka Desktop” and paste its token here.',
        einstellungen: 'Open settings in Horaka',
        token: 'Device token',
        platzhalter: '123|abc…',
        knopf: 'Connect',
        laeuft: 'Connecting…',
        hinweis: 'Access is kept in your system keychain, never in a file. Horaka Desktop needs the Premium plan.',
    },
    laden: {
        titel: 'Connecting to Horaka…',
        offline: 'Cannot reach Horaka. It carries on by itself as soon as your connection is back.',
        erneut: 'Try again now',
    },
    haupt: {
        suche: 'Search project, client or activity',
        laufend: 'Running',
        favoriten: 'Favourites',
        zuletzt: 'Recently used',
        projekte: 'All projects',
        keine: 'No projects found.',
        leer: 'No bookable projects yet. Add a client and a project in Horaka.',
        starten: 'Start',
        zusaetzlich: 'Start in addition',
        wechseln: 'Switch to this',
        stoppen: 'Stop',
        alleStoppen: 'Stop all',
        taetigkeiten: 'Show activities',
        ohneTaetigkeit: 'no activity',
        pausiert: 'paused',
        oeffnen: 'Open Horaka in the browser',
        einstellungen: 'Settings',
        schliessen: 'Close',
        notiz: 'Note',
        notizPlatzhalter: 'Add a note…',
        notizHinweis: 'Enter saves, Shift+Enter adds a line, Esc cancels.',
        speichern: 'Save',
        abbrechen: 'Cancel',
        kompakt: 'Compact view',
        breite: 'Change width (arrow keys)',
        gross: 'Full view',
    },
    abschluss: {
        titel: 'What did you do?',
        erfasst: ':dauer tracked',
        von: ':nr of :anzahl',
        platzhalter: 'A short summary, saved as the note of the entry',
        speichern: 'Save',
        ohne: 'Continue without a note',
        nichtMehr: 'Do not ask again',
        hinweis: 'Enter saves, Shift+Enter adds a line.',
    },
    einstellungen: {
        titel: 'Settings',
        zurueck: 'Back',
        autostart: 'Start when I sign in',
        autostartText: 'Horaka Desktop starts with your computer and waits in the background.',
        trennen: 'Disconnect',
        trennenText: 'Signs Horaka Desktop out and removes its access from the keychain. Connected apps and devices are listed in Horaka under API and devices.',
        version: 'Version',
        thema: 'Appearance',
        themen: { system: 'Like the system', hell: 'Light', dunkel: 'Dark' },
        notizen: 'Notes',
        notizBeimStoppen: 'Ask for a note when stopping',
        notizBeimStoppenText: 'After every stop, including by shortcut or menu, the window opens and asks what you did.',
        kuerzelTitel: 'Keyboard shortcuts',
        kuerzelText: 'Work everywhere, even with Horaka Desktop in the background. A notification tells you what happened.',
        kuerzelNeu: 'Add a shortcut',
        kuerzelAktion: 'What should happen?',
        kuerzelZiel: 'Project',
        kuerzelTaste: 'Key combination',
        kuerzelAufnehmen: 'Click and press the keys',
        kuerzelDruecken: 'Press the keys now…',
        kuerzelRegel: 'With Ctrl, Alt or Win (Mac: control, option or command), or a key F1 to F24.',
        kuerzelHinzufuegen: 'Add',
        kuerzelEntfernen: 'Remove',
        kuerzelDoppelt: 'This shortcut is already taken.',
        kuerzelBelegt: 'The system does not accept these shortcuts, probably another program uses them: :liste',
        kuerzelLeer: 'No shortcuts yet.',
        aktionen: {
            fenster: 'Open and close the window',
            notiz: 'Note for the running timer',
            stoppen: 'Stop all timers',
            umschalten: 'Start or stop a project',
            wechseln: 'Switch to a project',
        },
        projektWaehlen: 'Choose a project…',
        sprache: 'Language',
        spracheSystem: 'Same as the system',
        spracheText: 'The window reloads afterwards.',
        kompakt: 'Compact view',
        kompaktText: 'A small window with the running timers that stays open, even when you click elsewhere and after a restart.',
        durchsichtig: 'Fade out when the mouse is elsewhere',
        durchsichtigText: 'The small window turns see-through and comes back as soon as you move the mouse over it.',
        allgemein: 'General',
        kuerzelReiter: 'Shortcuts',
        system: 'System',
        deckkraft: 'Opacity when faded',
    },
    update: {
        da: 'Horaka Desktop :version is out.',
        installieren: 'Update now',
        laeuft: 'Updating, the app restarts in a moment…',
        titel: 'Updates',
        suchen: 'Check for updates at start',
        suchenText: 'And every 6 hours. A notification tells you about a new version; it is only installed when you want.',
        jetzt: 'Check for updates now',
        sucht: 'Checking…',
        aktuell: 'You have the latest version.',
    },
    fehler: {
        unauthorized: 'The token is no longer valid. Connect Horaka Desktop again.',
        insufficient_scope: 'Horaka Desktop lacks the “Trigger” permission. Disconnect in the settings and connect again with “Trigger” ticked.',
        oauth_scope_track: 'Without the “Trigger” permission Horaka Desktop cannot control timers. Connect again and tick “Trigger”.',
        network: 'Cannot reach Horaka. Check your connection.',
        rate_limited: 'Too many requests. Try again in a moment.',
        keychain: 'The keychain cannot be used.',
        token_missing: 'Paste a token first.',
        browser: 'The browser could not be opened.',
        oauth_timeout: 'Signing in took too long. Try again.',
        oauth_access_denied: 'You declined access.',
        oauth_state: 'The browser’s answer did not match this sign-in. Try again.',
        plan_required: 'Your plan does not allow this. Horaka Desktop needs Premium.',
        unknown: 'Something went wrong.',
        update: 'The update did not work. Try again later or download the new version from the website.',
    },
    tasten: {
        strg: 'Ctrl',
        umschalt: 'Shift',
        leertaste: 'Space',
    },
};

const alle: Record<string, Texte> = { de, en, fr, es, it, nl, pl, pt };

/** Namen der Sprachen in ihrer eigenen Sprache (Auswahl in den Einstellungen). */
export const SPRACHEN: Record<string, string> = {
    de: 'Deutsch',
    en: 'English',
    fr: 'Français',
    es: 'Español',
    it: 'Italiano',
    nl: 'Nederlands',
    pl: 'Polski',
    pt: 'Português',
};

function ermitteln(wahl: string): string {
    const code = (wahl === 'system' ? navigator.language : wahl).slice(0, 2).toLowerCase();
    return code in alle ? code : 'en';
}

/** Gewählte Sprache, bei `system` die des Systems, sonst Englisch. */
export let sprache = ermitteln('system');
export let t: Texte = alle[sprache];

/** Vor dem Start des Fensters (main.ts); ein Wechsel lädt das Fenster neu. */
export function spracheSetzen(wahl: string) {
    sprache = ermitteln(wahl);
    t = alle[sprache];
    document.documentElement.lang = sprache;
}

spracheSetzen('system');

/** Meldung zu einem Fehler: eigene Texte, sonst die der API (Sprache des Kontos). */
export function fehlertext(code: string, detail: string | null): string {
    const eigene = t.fehler as Record<string, string>;
    if (code in eigene) {
        return eigene[code];
    }
    return detail ?? eigene.unknown;
}

export const istMac = /Mac/i.test(navigator.userAgent);

/** Text mit Platzhaltern wie `:dauer` füllen. */
export function mit(text: string, werte: Record<string, string | number>): string {
    return text.replace(/:([a-z]+)/g, (alles, name: string) => (name in werte ? String(werte[name]) : alles));
}
