import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Aktion, Einstellungen, EinstellungenAntwort, Fehler, Stand } from './typen';

// Die Befehle aus src-tauri/src/lib.rs. Alles Netz läuft in Rust, das
// Fenster sieht nie den Token.

export const standLesen = () => invoke<Stand>('stand_lesen');
export const verbinden = (token: string) => invoke<Stand>('verbinden', { token });
export const anmelden = () => invoke<Stand>('anmelden');
export const anmeldungAbbrechen = () => invoke<void>('anmeldung_abbrechen');
export const trennen = () => invoke<Stand>('trennen');
export const neuLaden = () => invoke<Stand>('neu_laden');
export const verbergen = () => invoke<void>('verbergen');
export const groesseSetzen = (breite: number, hoehe: number) => invoke<void>('groesse_setzen', { breite, hoehe });
export const appOeffnen = (pfad?: string) => invoke<void>('app_oeffnen', { pfad: pfad ?? null });

export const notizSetzen = (id: number, notiz: string) => invoke<Stand>('notiz_setzen', { id, notiz });
export const eintragVerwerfen = (id: number) => invoke<Stand>('eintrag_verwerfen', { id });
export const abschlussUeberspringen = (id: number) => invoke<Stand>('abschluss_ueberspringen', { id });
export const einstellungenLesen = () => invoke<Einstellungen>('einstellungen_lesen');
export const einstellungenSpeichern = (neu: Einstellungen) => invoke<EinstellungenAntwort>('einstellungen_speichern', { neu });
export const updateSuchen = () => invoke<Stand>('update_suchen');
export const updateInstallieren = () => invoke<void>('update_installieren');
export const kuerzelPausieren = () => invoke<void>('kuerzel_pausieren');
export const kuerzelFortsetzen = () => invoke<string[]>('kuerzel_fortsetzen');

export const ausloesen = (aktion: Aktion, projekt: number | null = null, taetigkeit: number | null = null) =>
    invoke<Stand>('ausloesen', { aktion, projekt, taetigkeit });

export const aufStand = (fn: (stand: Stand) => void): Promise<UnlistenFn> => listen<Stand>('stand', (e) => fn(e.payload));
export const aufFehler = (fn: (fehler: Fehler) => void): Promise<UnlistenFn> => listen<Fehler>('fehler', (e) => fn(e.payload));
export const aufNotizFokus = (fn: () => void): Promise<UnlistenFn> => listen('notiz_fokus', () => fn());
export const aufMaus = (fn: (drueber: boolean) => void): Promise<UnlistenFn> => listen<boolean>('maus', (e) => fn(e.payload));
export const aufEinstellungenOeffnen = (fn: () => void): Promise<UnlistenFn> => listen('einstellungen_oeffnen', () => fn());
export const aufGezeigt = (fn: () => void): Promise<UnlistenFn> => listen('gezeigt', () => fn());

export function alsFehler(e: unknown): Fehler {
    if (e && typeof e === 'object' && 'code' in e) {
        return e as Fehler;
    }
    return { code: 'unknown', detail: String(e) };
}
