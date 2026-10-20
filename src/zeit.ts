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

/** Uhr, die jede Sekunde tickt (für laufende Timer). */
export function useUhr() {
    const jetzt = ref(Date.now());
    const takt = window.setInterval(() => (jetzt.value = Date.now()), 1000);
    onBeforeUnmount(() => window.clearInterval(takt));
    return jetzt;
}
