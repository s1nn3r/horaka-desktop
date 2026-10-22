import { ref } from 'vue';

// Eigene Reihenfolge der Kacheln je Ansicht (`name`), per Ziehen mit der
// Maus. Nur auf diesem Rechner und unabhängig von den Favoriten in Horaka;
// Neue stehen hinten.
// shortcut: nur in localStorage dieses Fensters, in einstellungen.json, falls sie mit umziehen soll.
export function useReihenfolge(name: string) {
    const schluessel = `reihenfolge-${name}`;
    const gemerkt = ref<string[]>(lesen());
    const ziehend = ref<string | null>(null);

    function lesen(): string[] {
        try {
            const wert: unknown = JSON.parse(localStorage.getItem(schluessel) ?? '[]');
            return Array.isArray(wert) ? wert.map(String) : [];
        } catch {
            return [];
        }
    }

    function ordnen<T extends { schluessel: string }>(liste: T[]): T[] {
        const rang = new Map(gemerkt.value.map((s, i) => [s, i]));
        return liste
            .map((k, i) => ({ k, r: rang.get(k.schluessel) ?? gemerkt.value.length + i }))
            .sort((a, b) => a.r - b.r)
            .map((x) => x.k);
    }

    // Erst ab 5 px Bewegung ist es Ziehen, sonst ein normaler Klick.
    let druck: { x: number; y: number; reihe: string[] } | null = null;

    function an(e: PointerEvent, eigener: string, reihe: string[]) {
        if (e.button !== 0) {
            return;
        }
        druck = { x: e.clientX, y: e.clientY, reihe };
        const bewegen = (m: PointerEvent) => {
            if (!druck || (ziehend.value === null && Math.hypot(m.clientX - druck.x, m.clientY - druck.y) < 5)) {
                return;
            }
            ziehend.value = eigener;
            const ziel = document.elementFromPoint(m.clientX, m.clientY)?.closest<HTMLElement>('[data-schluessel]')?.dataset.schluessel;
            const nach = ziel === undefined ? -1 : druck.reihe.indexOf(ziel);
            if (ziel !== eigener && nach >= 0) {
                const neu = druck.reihe.filter((s) => s !== eigener);
                neu.splice(nach, 0, eigener);
                druck.reihe = neu;
                gemerkt.value = neu;
            }
        };
        const los = () => {
            window.removeEventListener('pointermove', bewegen);
            window.removeEventListener('pointerup', los);
            window.removeEventListener('pointercancel', los);
            if (ziehend.value !== null) {
                gezogenUm = Date.now();
                try {
                    localStorage.setItem(schluessel, JSON.stringify(gemerkt.value));
                } catch {
                    // Ohne Speicher gilt die Reihenfolge nur bis zum Neustart.
                }
            }
            ziehend.value = null;
            druck = null;
        };
        window.addEventListener('pointermove', bewegen);
        window.addEventListener('pointerup', los);
        window.addEventListener('pointercancel', los);
    }

    return { ordnen, an, ziehend };
}

// Der Klick, der auf das Loslassen nach dem Ziehen folgt, startet nichts.
let gezogenUm = 0;
window.addEventListener(
    'click',
    (e) => {
        if (Date.now() - gezogenUm < 300) {
            e.stopPropagation();
            e.preventDefault();
        }
    },
    true,
);
