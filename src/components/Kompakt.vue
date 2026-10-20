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
                class="kachel-laufend kachel-streifen flex h-8 items-center gap-2 rounded-md pr-1 pl-3.5"
                :style="{ '--farbe': l.projekt?.color ?? 'var(--primary)' }"
                :title="titel(l.projekt, l.taetigkeit)"
            >
                <span class="min-w-0 flex-1 truncate text-[12px]">{{ l.projekt?.name ?? `#${l.eintrag.project_id}` }}</span>
                <span class="zeit-laufend text-[12px] font-semibold tabular-nums">{{ dauer(seit(l.eintrag.started_at, jetzt)) }}</span>
                <button
                    type="button"
                    class="flex size-6 shrink-0 items-center justify-center rounded text-destructive hover:bg-foreground/10"
                    :disabled="beschaeftigt"
                    :title="t.haupt.stoppen"
                    @click="aktion('stop', l.eintrag.project_id)"
                >
                    <svg viewBox="0 0 16 16" class="size-2.5" aria-hidden="true"><rect x="3" y="3" width="10" height="10" rx="2" fill="currentColor" /></svg>
                </button>
            </div>

            <!-- Favoriten als Kacheln wie Windows 11: Farbstreifen links, Name, darunter Tätigkeit (oder Kunde); laufend zart in der Projektfarbe getönt und umrandet, mit Dauer. -->
            <div v-if="kacheln.length > 0" data-tauri-drag-region class="grid grid-cols-[repeat(auto-fill,minmax(7rem,1fr))] gap-2">
                <button
                    v-for="k in kacheln"
                    :key="k.schluessel"
                    type="button"
                    class="kachel-streifen flex h-12 min-w-0 flex-col justify-center gap-0.5 rounded-md pr-2.5 pl-3.5 text-left"
                    :class="k.lauf ? 'kachel-laufend' : 'bg-kachel shadow-[inset_0_0_0_var(--haarlinie)_var(--kante)] hover:bg-kachel-hover active:text-foreground/70'"
                    :style="{ '--farbe': k.projekt.color }"
                    :disabled="beschaeftigt"
                    :title="`${titel(k.projekt, k.taetigkeit)} · ${k.lauf ? t.haupt.stoppen : t.haupt.starten}`"
                    @click="aktion(k.lauf ? 'stop' : 'start', k.projekt.id, k.taetigkeit?.id ?? null)"
                >
                    <span class="truncate text-[12px] leading-4">{{ k.projekt.name }}</span>
                    <span v-if="k.lauf" class="zeit-laufend text-[11px] leading-4 font-semibold tabular-nums">{{ dauer(seit(k.lauf.eintrag.started_at, jetzt)) }}</span>
                    <span v-else class="truncate text-[11px] leading-4 text-muted-foreground">{{ k.taetigkeit?.name ?? k.projekt.client_name }}</span>
                </button>
            </div>
        </div>

        <!-- Statt Kopfzeile: Schließen, volle Ansicht, im Browser öffnen. Die Spalte
             ist so hoch wie die Kacheln (bei einer Reihe 48 px, oben und unten bündig),
             höchstens 96 px, damit die Symbole bei mehreren Reihen nicht zu weit auseinanderstehen. -->
        <div data-tauri-drag-region class="flex max-h-24 w-6 shrink-0 flex-col">
            <!-- Symbole 12 px im 12er Raster, damit sie auf ganze Pixel fallen. -->
            <button type="button" class="flex flex-1 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground" :title="t.haupt.schliessen" @click="verbergen()">
                <svg viewBox="0 0 12 12" class="size-3" aria-hidden="true"><path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" /></svg>
            </button>
            <button type="button" class="flex flex-1 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground" :title="t.haupt.gross" @click="emit('gross')">
                <!-- Vergrößern (Form wie lucide „maximize-2“). -->
                <svg viewBox="0 0 12 12" class="size-3" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                    <path d="M7.5 1.5h3v3M4.5 10.5h-3v-3M10.5 1.5L7 5M1.5 10.5L5 7" />
                </svg>
            </button>
            <button type="button" class="flex flex-1 items-center justify-center rounded text-muted-foreground hover:bg-foreground/10 hover:text-foreground" :title="t.haupt.oeffnen" @click="appOeffnen()">
                <svg viewBox="0 0 12 12" class="size-3" aria-hidden="true">
                    <path d="M7 1.5h3.5V5M10.5 1.5L6 6M8.5 7v3.5h-7v-7H5" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
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
