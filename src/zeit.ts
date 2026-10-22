import { onBeforeUnmount, ref } from 'vue';

/** Sekunden seit einem Zeitpunkt der API, nie negativ. */
export function seit(iso: string, jetzt: number): number {
    return Math.max(0, Math.floor((jetzt - Date.parse(iso)) / 1000));
}

/** Dauer als „1:05:09“ (unter einer Stunde „5:09“). */
export function dauer(sekunden: number): string {
    const h = Math.floor(sekunden / 3600);
    const m = Math.floor((sekunden % 3600) / 60);
    const s = sekunden % 60;
    const zwei = (n: number) => String(n).padStart(2, '0');
    return h > 0 ? `${h}:${zwei(m)}:${zwei(s)}` : `${m}:${zwei(s)}`;
}

/**
 * Uhrzeit „HH:MM“ als letzter solcher Zeitpunkt bis jetzt (Ortszeit), für
 * den Start eines laufenden Timers: 23:50 kurz nach Mitternacht ist gestern.
 */
export function letzteUhrzeit(uhrzeit: string, jetzt: Date): Date | null {
    const teile = /^(\d{1,2}):(\d{2})$/.exec(uhrzeit);
    if (!teile) {
        return null;
    }
    const d = new Date(jetzt);
    d.setHours(Number(teile[1]), Number(teile[2]), 0, 0);
    if (d.getTime() > jetzt.getTime()) {
        d.setDate(d.getDate() - 1);
    }
    return d;
}

/** „HH:MM“ eines Zeitpunkts der API in Ortszeit, für das Zeitfeld. */
export function uhrzeit(iso: string): string {
    const d = new Date(iso);
    return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

/** Uhr, die jede Sekunde tickt (für laufende Timer). */
export function useUhr() {
    const jetzt = ref(Date.now());
    const takt = window.setInterval(() => (jetzt.value = Date.now()), 1000);
    onBeforeUnmount(() => window.clearInterval(takt));
    return jetzt;
}
