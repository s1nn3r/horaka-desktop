<script setup lang="ts">
import { computed, ref } from 'vue';
import { alsFehler, appOeffnen, ausloesen, verbergen } from '../befehle';
import { t } from '../i18n';
import type { Aktion, Auswahl, Fehler, Kombination, Projekt, Stand, Taetigkeit, Timer } from '../typen';
import { dauer, seit, useUhr } from '../zeit';

// Kompaktansicht: kleines Fenster, das offen bleibt. Alle Favoriten als
// kleine quadratische Kacheln (ohne Favoriten die zuletzt verwendeten), auch
// während ein Timer läuft. Die Kachel selbst startet und stoppt und zeigt,
// ob sie läuft. Ein laufender Timer ohne eigene Kachel steht darüber als
// schmale Zeile. Ohne Kopfzeile: Symbole rechts in einer schmalen Spalte,
// gezogen wird am grauen Rand (Optik wie die Flyouts von Windows 11).
// Die Höhe folgt dem Inhalt (App.vue).
const props = defineProps<{ auswahl: Auswahl; timer: Timer | null; offline: boolean; spalten: number }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler]; gross: []; spalten: [anzahl: number]; ziehen: [an: boolean] }>();

// Griff am rechten Rand: Ziehen ändert die Spaltenzahl (je 120 px eine),
// App.vue setzt daraus Breite und Höhe.
let ziehenAb: { x: number; spalten: number } | null = null;
function griffAn(e: PointerEvent) {
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    ziehenAb = { x: e.screenX, spalten: props.spalten };
    emit('ziehen', true);
}
function griffZiehen(e: PointerEvent) {
    if (ziehenAb) {
        const neu = ziehenAb.spalten + Math.round((e.screenX - ziehenAb.x) / 120);
        if (neu !== props.spalten) {
            emit('spalten', neu);
        }
    }
}
function griffAus() {
    ziehenAb = null;
    emit('ziehen', false);
}

const jetzt = useUhr();
const beschaeftigt = ref(false);
const projekte = computed(() => new Map(props.auswahl.projects.map((p) => [p.id, p])));

function taetigkeitVon(projekt: Projekt | undefined, id: number | null): Taetigkeit | null {
    return id === null ? null : (projekt?.activities.find((a) => a.id === id) ?? null);
}

const laufende = computed(() =>
    (props.timer?.entries ?? []).map((e) => {
        const projekt = projekte.value.get(e.project_id);
        return { eintrag: e, projekt: projekt ?? null, taetigkeit: taetigkeitVon(projekt, e.activity_id) };
    }),
);

const kacheln = computed(() => {
    const quelle: Kombination[] = props.auswahl.favorites.length > 0 ? props.auswahl.favorites : props.auswahl.recent.slice(0, 8);
    const gesehen = new Set<string>();
    return quelle.flatMap((k) => {
        const projekt = projekte.value.get(k.project_id);
        const schluessel = `${k.project_id}:${k.activity_id ?? 0}`;
        if (!projekt || gesehen.has(schluessel)) {
            return [];
        }
        gesehen.add(schluessel);
        const taetigkeit = taetigkeitVon(projekt, k.activity_id);
        // Läuft die Kombination (ohne Tätigkeit: irgendetwas im Projekt)?
        const lauf = laufende.value.find(
            (l) => l.eintrag.project_id === projekt.id && (taetigkeit === null || l.eintrag.activity_id === taetigkeit.id),
        );
        return [{ schluessel, projekt, taetigkeit, lauf }];
    });
});

// Laufende Timer, die keine Kachel zeigt.
const ohneKachel = computed(() => {
    const gezeigt = new Set(kacheln.value.flatMap((k) => (k.lauf ? [k.lauf.eintrag.id] : [])));
    return laufende.value.filter((l) => !gezeigt.has(l.eintrag.id));
});

// Schrift auf der Projektfarbe: schwarz oder weiß, je nachdem, was besser lesbar ist.
function schriftAuf(farbe: string | undefined): string {
    const m = /^#?([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(farbe ?? '');
    if (!m) {
        return '#ffffff';
    }
    const [r, g, b] = m.slice(1).map((h) => {
        const c = parseInt(h, 16) / 255;
        return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b > 0.18 ? '#000000' : '#ffffff';
}

function titel(projekt: Projekt | null, taetigkeit: Taetigkeit | null): string {
    return [projekt?.client_name, projekt?.name, taetigkeit?.name].filter(Boolean).join(' · ');
}

async function aktion(was: Aktion, projekt: number | null = null, taetigkeit: number | null = null) {
    if (beschaeftigt.value) {
        return;
    }
    beschaeftigt.value = true;
    try {
        emit('stand', await ausloesen(was, projekt, taetigkeit));
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        beschaeftigt.value = false;
    }
}
</script>

<template>
    <div data-tauri-drag-region class="relative flex cursor-default gap-2 p-3">
        <div data-tauri-drag-region class="flex min-w-0 flex-1 flex-col gap-2">
            <div v-if="offline" role="status" class="px-0.5 text-[11px] leading-tight text-muted-foreground">
                {{ t.laden.offline }}
            </div>

            <!-- Laufende Timer ohne eigene Kachel: eine schmale Zeile je Timer. -->
            <div
                v-for="l in ohneKachel"
                :key="l.eintrag.id"
                class="flex h-8 items-center gap-2 rounded-md pr-1 pl-2.5"
                :style="{ background: l.projekt?.color ?? 'var(--primary)', color: schriftAuf(l.projekt?.color) }"
                :title="titel(l.projekt, l.taetigkeit)"
            >
                <span class="min-w-0 flex-1 truncate text-[12px]">{{ l.projekt?.name ?? `#${l.eintrag.project_id}` }}</span>
                <span class="text-[12px] font-semibold tabular-nums">{{ dauer(seit(l.eintrag.started_at, jetzt)) }}</span>
                <button
                    type="button"
                    class="flex size-6 shrink-0 items-center justify-center rounded hover:bg-black/15"
                    :disabled="beschaeftigt"
                    :title="t.haupt.stoppen"
                    @click="aktion('stop', l.eintrag.project_id)"
                >
                    <svg viewBox="0 0 16 16" class="size-2.5" aria-hidden="true"><rect x="3" y="3" width="10" height="10" rx="2" fill="currentColor" /></svg>
                </button>
            </div>

            <!-- Favoriten als Kacheln wie Windows 11: Name, darunter Farbe und Tätigkeit (oder Kunde), laufend in der Projektfarbe mit Dauer. -->
            <div v-if="kacheln.length > 0" data-tauri-drag-region class="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-2">
                <button
                    v-for="k in kacheln"
                    :key="k.schluessel"
                    type="button"
                    class="flex h-12 min-w-0 flex-col justify-center gap-0.5 rounded-md px-2.5 text-left"
                    :class="k.lauf ? '' : 'bg-kachel shadow-[inset_0_0_0_var(--haarlinie)_var(--muted-foreground)] hover:bg-kachel-hover active:text-foreground/70'"
                    :style="k.lauf ? { background: k.projekt.color, color: schriftAuf(k.projekt.color) } : undefined"
                    :disabled="beschaeftigt"
                    :title="`${titel(k.projekt, k.taetigkeit)} · ${k.lauf ? t.haupt.stoppen : t.haupt.starten}`"
                    @click="aktion(k.lauf ? 'stop' : 'start', k.projekt.id, k.taetigkeit?.id ?? null)"
                >
                    <span class="truncate text-[12px] leading-4">{{ k.projekt.name }}</span>
                    <span v-if="k.lauf" class="text-[11px] leading-4 font-semibold tabular-nums opacity-90">{{ dauer(seit(k.lauf.eintrag.started_at, jetzt)) }}</span>
                    <span v-else class="flex min-w-0 items-center gap-1.5 text-[11px] leading-4 text-muted-foreground">
                        <span class="size-2 shrink-0 rounded-[2px]" :style="{ background: k.projekt.color }" />
                        <span class="truncate">{{ k.taetigkeit?.name ?? k.projekt.client_name }}</span>
                    </span>
                </button>
            </div>
        </div>

        <!-- Statt Kopfzeile: Schließen, volle Ansicht, im Browser öffnen. -->
        <div data-tauri-drag-region class="flex shrink-0 flex-col gap-0.5">
            <!-- Symbole 16 px im 16er Raster, damit sie auf ganze Pixel fallen. -->
            <button type="button" class="flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground" :title="t.haupt.schliessen" @click="verbergen()">
                <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
            </button>
            <button type="button" class="flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground" :title="t.haupt.gross" @click="emit('gross')">
                <!-- Vergrößern (Form wie lucide „maximize-2“). -->
                <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M10 2.5h3.5V6M6 13.5H2.5V10M13.5 2.5L9 7M2.5 13.5L7 9" />
                </svg>
            </button>
            <button type="button" class="flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground" :title="t.haupt.oeffnen" @click="appOeffnen()">
                <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true">
                    <path d="M9 3h4v4M13 3L7.5 8.5M11 9.5V13H3V5h3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                </svg>
            </button>
        </div>

        <div
            role="separator"
            aria-orientation="vertical"
            tabindex="0"
            :aria-label="t.haupt.breite"
            :aria-valuenow="spalten"
            class="absolute inset-y-0 right-0 w-1.5 cursor-ew-resize focus-visible:bg-ring/40"
            @pointerdown="griffAn"
            @pointermove="griffZiehen"
            @pointerup="griffAus"
            @pointercancel="griffAus"
            @lostpointercapture="griffAus"
            @keydown.left.prevent="emit('spalten', spalten - 1)"
            @keydown.right.prevent="emit('spalten', spalten + 1)"
        />
    </div>
</template>
