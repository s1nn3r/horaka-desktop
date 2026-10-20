import { istMac, t } from './i18n';

// Tastenkürzel als Text für das Betriebssystem (`control+alt+KeyH`, so wie
// ihn Shortcut::from_str in Rust versteht) und als Anzeige (⌃⌥H am Mac,
// Strg+Alt+H unter Windows).

const NUR_MODIFIKATOR = /^(Control|Alt|Shift|Meta|OS)(Left|Right)?$/;
const F_TASTE = /^F([1-9]|1[0-9]|2[0-4])$/;

/** Kürzel aus einem Tastendruck, null wenn es (noch) keins ist. */
export function ausTaste(e: KeyboardEvent): string | null {
    if (NUR_MODIFIKATOR.test(e.code) || e.code === '') {
        return null;
    }
    const teile: string[] = [];
    if (e.ctrlKey) teile.push('control');
    if (e.altKey) teile.push('alt');
    if (e.shiftKey) teile.push('shift');
    if (e.metaKey) teile.push('super');

    // Ohne Strg, Alt oder Win/cmd nur F-Tasten, sonst fängt das Kürzel
    // normales Tippen ab.
    const mitModifikator = e.ctrlKey || e.altKey || e.metaKey;
    if (!mitModifikator && !F_TASTE.test(e.code)) {
        return null;
    }
    teile.push(e.code);
    return teile.join('+');
}

function tastenname(code: string): string {
    if (code.startsWith('Key')) return code.slice(3);
    if (code.startsWith('Digit')) return code.slice(5);
    if (code.startsWith('Numpad')) return `Num ${code.slice(6)}`;
    const namen: Record<string, string> = {
        Space: t.tasten.leertaste,
        ArrowUp: '↑',
        ArrowDown: '↓',
        ArrowLeft: '←',
        ArrowRight: '→',
        Enter: '↵',
        Backspace: '⌫',
        Escape: 'Esc',
        Minus: '-',
        Equal: '=',
        Comma: ',',
        Period: '.',
        Slash: '/',
        Backslash: '\\',
        Semicolon: ';',
        Quote: "'",
        BracketLeft: '[',
        BracketRight: ']',
        Backquote: '`',
    };
    return namen[code] ?? code;
}

/** Anzeige eines Kürzels. */
export function anzeigen(kuerzel: string): string {
    const teile = kuerzel.split('+');
    const taste = tastenname(teile.pop() ?? '');
    const mod = new Set(teile.map((t) => t.toLowerCase()));
    if (istMac) {
        return [mod.has('control') ? '⌃' : '', mod.has('alt') ? '⌥' : '', mod.has('shift') ? '⇧' : '', mod.has('super') ? '⌘' : '', taste].join('');
    }
    const namen = { control: t.tasten.strg, shift: t.tasten.umschalt };
    return [
        mod.has('control') ? namen.control : null,
        mod.has('alt') ? 'Alt' : null,
        mod.has('shift') ? namen.shift : null,
        mod.has('super') ? 'Win' : null,
        taste,
    ]
        .filter(Boolean)
        .join('+');
}

/** Gleiches Kürzel, egal in welcher Reihenfolge und Schreibweise. */
export function gleich(a: string, b: string): boolean {
    const norm = (k: string) => k.split('+').map((t) => t.toLowerCase().replace(/^ctrl$/, 'control')).sort().join('+');
    return norm(a) === norm(b);
}
