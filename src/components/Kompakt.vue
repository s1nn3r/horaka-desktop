<script setup lang="ts">
import { computed, ref } from 'vue';
import { alsFehler, appOeffnen, ausloesen, verbergen } from '../befehle';
import { sprache, t } from '../i18n';
import type { Aktion, Auswahl, Fehler, Kombination, Projekt, Stand, Taetigkeit, Timer } from '../typen';
import { dauer, seit, useUhr } from '../zeit';
import Zeichen from './Zeichen.vue';

// Kompaktansicht: kleines Fenster, das offen bleibt. Alle Favoriten als
// kleine quadratische Kacheln (ohne Favoriten die zuletzt verwendeten), auch
// während ein Timer läuft. Die Kachel selbst startet und stoppt und zeigt,
// ob sie läuft. Ein laufender Timer ohne eigene Kachel steht darüber als
// schmale Zeile. Die Höhe folgt dem Inhalt (App.vue).
const props = defineProps<{ auswahl: Auswahl; timer: Timer | null; offline: boolean }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler]; gross: [] }>();

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
    <header data-tauri-drag-region class="flex cursor-default items-center gap-1 border-b border-border py-0.5 pr-0.5 pl-1.5">
        <Zeichen class="pointer-events-none size-4" />
        <span data-tauri-drag-region class="flex-1 truncate text-[11px] font-semibold text-foreground/80">
            {{ laufende.length > 0 ? t.haupt.laufend : t.haupt.nichtsLaeuft }}
        </span>
        <button type="button" class="flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-muted" :title="t.haupt.oeffnen" @click="appOeffnen()">
            <svg viewBox="0 0 16 16" class="size-3" aria-hidden="true">
                <path d="M9 3h4v4M13 3L7.5 8.5M11 9.5V13H3V5h3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>
        <button type="button" class="flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-muted" :title="t.haupt.gross" @click="emit('gross')">
            <!-- Vergrößern (Form wie lucide „maximize-2“). -->
            <svg viewBox="0 0 24 24" class="size-3" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7" />
            </svg>
        </button>
        <button type="button" class="flex size-6 items-center justify-center rounded text-muted-foreground hover:bg-muted" :title="t.haupt.schliessen" @click="verbergen()">
            <svg viewBox="0 0 16 16" class="size-3" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
        </button>
    </header>

    <div v-if="offline" role="status" class="border-b border-border bg-muted px-2 py-1 text-[11px] text-muted-foreground">
        {{ t.laden.offline }}
    </div>

    <!-- Laufende Timer ohne eigene Kachel: eine schmale Zeile je Timer. -->
    <div
        v-for="l in ohneKachel"
        :key="l.eintrag.id"
        class="flex h-8 items-center gap-1.5 border-b border-border bg-primary-tint pr-1 pl-2"
        :title="titel(l.projekt, l.taetigkeit)"
    >
        <span class="size-2 shrink-0 animate-pulse rounded-full" :style="{ background: l.projekt?.color ?? 'currentColor' }" />
        <span class="min-w-0 flex-1 truncate text-[12px] font-medium">{{ l.projekt?.name ?? `#${l.eintrag.project_id}` }}</span>
        <span class="font-mono text-[12px] font-semibold tabular-nums">{{ dauer(seit(l.eintrag.started_at, jetzt)) }}</span>
        <button
            type="button"
            class="flex size-6 shrink-0 items-center justify-center rounded-full bg-destructive text-destructive-foreground"
            :disabled="beschaeftigt"
            :title="t.haupt.stoppen"
            @click="aktion('stop', l.eintrag.project_id)"
        >
            <svg viewBox="0 0 16 16" class="size-2.5" aria-hidden="true"><rect x="3.5" y="3.5" width="9" height="9" rx="1.5" fill="currentColor" /></svg>
        </button>
    </div>

    <!-- Favoriten als quadratische Kacheln, so viele nebeneinander wie passen. -->
    <div v-if="kacheln.length > 0" class="grid grid-cols-[repeat(auto-fill,minmax(3.25rem,1fr))] gap-1 p-1">
        <button
            v-for="k in kacheln"
            :key="k.schluessel"
            type="button"
            class="relative flex aspect-square min-w-0 flex-col justify-between overflow-hidden rounded-md border p-1 text-left transition-colors"
            :class="k.lauf ? 'border-primary bg-primary-tint' : 'border-muted-foreground/35 bg-muted/50 hover:bg-muted'"
            :disabled="beschaeftigt"
            :title="`${titel(k.projekt, k.taetigkeit)} · ${k.lauf ? t.haupt.stoppen : t.haupt.starten}`"
            @click="aktion(k.lauf ? 'stop' : 'start', k.projekt.id, k.taetigkeit?.id ?? null)"
        >
            <span class="absolute inset-x-0 top-0 h-1" :style="{ background: k.projekt.color }" />
            <!-- Zustand: Stopp-Zeichen, wenn sie läuft, sonst Start. -->
            <svg v-if="k.lauf" viewBox="0 0 16 16" class="absolute right-1 bottom-1 size-2.5 text-destructive" aria-hidden="true">
                <rect x="3" y="3" width="10" height="10" rx="2" fill="currentColor" />
            </svg>
            <svg v-else viewBox="0 0 16 16" class="absolute right-1 bottom-1 size-2.5 text-foreground/70" aria-hidden="true">
                <path d="M4.5 2.8v10.4L13 8z" fill="currentColor" />
            </svg>
            <span class="mt-0.5 line-clamp-2 text-[10px] leading-tight font-medium hyphens-auto [overflow-wrap:anywhere]" :lang="sprache">{{ k.projekt.name }}</span>
            <span v-if="k.lauf" class="pr-3 font-mono text-[9px] font-semibold text-primary tabular-nums">{{ dauer(seit(k.lauf.eintrag.started_at, jetzt)) }}</span>
            <span v-else-if="k.taetigkeit" class="truncate pr-3 text-[9px] leading-tight text-foreground/75">{{ k.taetigkeit.name }}</span>
        </button>
    </div>
</template>
