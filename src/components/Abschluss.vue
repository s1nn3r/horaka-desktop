<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { abschlussUeberspringen, alsFehler, notizSetzen } from '../befehle';
import { projektfarbe } from '../farbe';
import { mit, t } from '../i18n';
import type { Auswahl, Fehler, Gestoppt, Stand } from '../typen';
import { dauer } from '../zeit';
import Kopf from './Kopf.vue';

// Nach dem Stoppen: „Was hast du gemacht?“ für jeden gestoppten Eintrag
// nacheinander, vorbefüllt mit der Notiz, die er schon hatte.
const props = defineProps<{ abschluss: Gestoppt[]; auswahl: Auswahl | null }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler]; einstellungen: []; nichtMehrFragen: [] }>();

const aktuell = computed(() => props.abschluss[0]);
const anzahl = ref(props.abschluss.length);
const text = ref('');
const speichert = ref(false);
const feld = ref<HTMLTextAreaElement | null>(null);

const projekt = computed(() => props.auswahl?.projects.find((p) => p.id === aktuell.value?.project_id) ?? null);
const taetigkeit = computed(() => projekt.value?.activities.find((a) => a.id === aktuell.value?.activity_id) ?? null);
const nummer = computed(() => anzahl.value - props.abschluss.length + 1);

function vorbereiten() {
    text.value = aktuell.value?.note ?? '';
    void nextTick(() => feld.value?.focus());
}

watch(() => aktuell.value?.id, vorbereiten);
watch(
    () => props.abschluss.length,
    (neu, alt) => {
        // Kommt ein weiterer dazu, zählt er mit.
        if (neu > alt) anzahl.value += neu - alt;
    },
);
onMounted(vorbereiten);

async function weiter(speichern: boolean) {
    const eintrag = aktuell.value;
    if (!eintrag || speichert.value) {
        return;
    }
    speichert.value = true;
    try {
        emit('stand', speichern ? await notizSetzen(eintrag.id, text.value) : await abschlussUeberspringen(eintrag.id));
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        speichert.value = false;
    }
}

function taste(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        void weiter(true);
    }
}
</script>

<template>
    <Kopf @einstellungen="emit('einstellungen')" />
    <main v-if="aktuell" class="flex flex-1 flex-col gap-3 overflow-y-auto p-4">
        <div class="flex items-baseline justify-between gap-2">
            <h1 class="text-base font-semibold">{{ t.abschluss.titel }}</h1>
            <span v-if="anzahl > 1" class="text-[12px] text-muted-foreground">{{ mit(t.abschluss.von, { nr: nummer, anzahl }) }}</span>
        </div>

        <div class="flex items-center gap-2 rounded-lg bg-muted px-3 py-2">
            <span class="size-2.5 shrink-0 rounded-full" :style="{ background: projektfarbe(projekt?.color) }" />
            <div class="min-w-0 flex-1">
                <div class="truncate font-medium">{{ projekt?.name ?? `#${aktuell.project_id}` }}</div>
                <div class="truncate text-[12px] text-muted-foreground">{{ [projekt?.client_name, taetigkeit?.name].filter(Boolean).join(' · ') }}</div>
            </div>
            <span class="font-mono text-[13px] font-semibold tabular-nums">{{ mit(t.abschluss.erfasst, { dauer: dauer(aktuell.sekunden) }) }}</span>
        </div>

        <textarea
            ref="feld"
            v-model="text"
            rows="5"
            maxlength="2000"
            :placeholder="t.abschluss.platzhalter"
            :aria-label="t.abschluss.titel"
            :disabled="speichert"
            class="w-full resize-none rounded-lg border border-border bg-background px-3 py-2 outline-none focus:border-primary"
            @keydown="taste"
        />
        <p class="-mt-1 text-[11px] text-muted-foreground">{{ t.abschluss.hinweis }}</p>

        <button type="button" class="h-10 rounded-lg bg-primary font-medium text-primary-foreground disabled:opacity-50" :disabled="speichert" @click="weiter(true)">
            {{ t.abschluss.speichern }}
        </button>
        <button type="button" class="h-10 rounded-lg border border-border font-medium hover:bg-muted disabled:opacity-50" :disabled="speichert" @click="weiter(false)">
            {{ t.abschluss.ohne }}
        </button>
        <button type="button" class="mt-auto self-center text-[12px] text-muted-foreground underline-offset-2 hover:underline" @click="emit('nichtMehrFragen')">
            {{ t.abschluss.nichtMehr }}
        </button>
    </main>
</template>
