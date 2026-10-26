<script setup lang="ts">
import { nextTick, onMounted, ref } from 'vue';
import { t } from '../i18n';
import type { Auswahl, Eintrag } from '../typen';
import { dauer } from '../zeit';

// Kürzel „Notiz“ mit mehreren laufenden Timern: welcher? Pfeiltasten
// wählen, Enter übernimmt, Esc schließt (App.vue).
const props = defineProps<{ eintraege: Eintrag[]; auswahl: Auswahl | null }>();
const emit = defineEmits<{ waehlen: [eintrag: Eintrag] }>();

const knoepfe = ref<HTMLButtonElement[]>([]);

function projekt(e: Eintrag) {
    return props.auswahl?.projects.find((p) => p.id === e.project_id) ?? null;
}
function unterzeile(e: Eintrag) {
    const p = projekt(e);
    return [p?.client_name, p?.activities.find((a) => a.id === e.activity_id)?.name].filter(Boolean).join(' · ');
}
const sekunden = (e: Eintrag) => Math.floor((Date.now() - Date.parse(e.started_at)) / 1000);

function pfeil(i: number) {
    const n = props.eintraege.length;
    knoepfe.value[(i + n) % n]?.focus();
}

// Der zuletzt gestartete steht unten und ist vorausgewählt.
onMounted(() => void nextTick(() => pfeil(props.eintraege.length - 1)));
</script>

<template>
    <main class="flex flex-col gap-2 p-4">
        <h1 class="text-base font-semibold">{{ t.abschluss.welcher }}</h1>
        <button
            v-for="(e, i) in eintraege"
            :key="e.id"
            ref="knoepfe"
            type="button"
            class="flex min-h-11 items-center gap-2 rounded-lg px-3 py-2 text-left outline-none hover:bg-muted focus-visible:bg-muted focus-visible:ring-2 focus-visible:ring-ring"
            @click="emit('waehlen', e)"
            @keydown.down.prevent="pfeil(i + 1)"
            @keydown.up.prevent="pfeil(i - 1)"
        >
            <span class="size-2.5 shrink-0 rounded-full" :style="{ background: projekt(e)?.color ?? 'currentColor' }" />
            <span class="min-w-0 flex-1">
                <span class="block truncate font-medium">{{ projekt(e)?.name ?? `#${e.project_id}` }}</span>
                <span class="block truncate text-[12px] text-muted-foreground">{{ e.note || unterzeile(e) }}</span>
            </span>
            <span class="font-mono text-[13px] tabular-nums">{{ dauer(sekunden(e)) }}</span>
        </button>
    </main>
</template>
