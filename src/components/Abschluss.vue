<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { abschlussUeberspringen, alsFehler, notizSetzen } from '../befehle';
import { mit, t } from '../i18n';
import type { Auswahl, Fehler, Gestoppt, Stand } from '../typen';
import { dauer, letzteUhrzeit, uhrzeit } from '../zeit';
import Kopf from './Kopf.vue';
import Vorschlaege from './Vorschlaege.vue';

// Nach dem Stoppen: „Was hast du gemacht?“ für jeden gestoppten Eintrag
// nacheinander, vorbefüllt mit der Notiz, die er schon hatte. Mit `laufend`
// dasselbe Fenster für die Notiz eines laufenden Timers (Kontextmenü der
// Kompaktansicht): Abbrechen statt „Ohne Notiz weiter“. Nach dem Stoppen
// stehen Start und Ende darüber und lassen sich korrigieren; gesendet wird
// nur, was sich geändert hat, der Server rundet nach den Regeln.
const props = defineProps<{ abschluss: Gestoppt[]; auswahl: Auswahl | null; laufend?: boolean }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler]; einstellungen: []; nichtMehrFragen: []; abbrechen: [] }>();

const aktuell = computed(() => props.abschluss[0]);
const anzahl = ref(props.abschluss.length);
const text = ref('');
const startWert = ref('');
const endeWert = ref('');
const speichert = ref(false);
const feld = ref<HTMLTextAreaElement | null>(null);

const projekt = computed(() => props.auswahl?.projects.find((p) => p.id === aktuell.value?.project_id) ?? null);
const taetigkeit = computed(() => projekt.value?.activities.find((a) => a.id === aktuell.value?.activity_id) ?? null);
const nummer = computed(() => anzahl.value - props.abschluss.length + 1);

// Ende als letztes Vorkommen bis jetzt, Start als letztes bis zum Ende:
// über Mitternacht liegt der Start am Vortag.
const zeiten = computed(() => {
    const ende = letzteUhrzeit(endeWert.value, new Date());
    const start = ende && letzteUhrzeit(startWert.value, ende);
    return start && ende ? { start, ende } : null;
});
const sekunden = computed(() => (zeiten.value ? Math.round((zeiten.value.ende.getTime() - zeiten.value.start.getTime()) / 1000) : (aktuell.value?.sekunden ?? 0)));

function vorbereiten() {
    text.value = aktuell.value?.note ?? '';
    startWert.value = aktuell.value ? uhrzeit(aktuell.value.started_at) : '';
    endeWert.value = aktuell.value ? uhrzeit(aktuell.value.ended_at) : '';
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
    if (props.laufend && !speichern) {
        emit('abbrechen');
        return;
    }
    let start: string | undefined;
    let ende: string | undefined;
    if (!props.laufend) {
        if (!zeiten.value) {
            return;
        }
        if (startWert.value !== uhrzeit(eintrag.started_at)) start = zeiten.value.start.toISOString();
        if (endeWert.value !== uhrzeit(eintrag.ended_at)) ende = zeiten.value.ende.toISOString();
    }
    // Auch „Ohne Notiz weiter“ übernimmt geänderte Zeiten, die Notiz bleibt dann.
    const mitZeiten = start !== undefined || ende !== undefined;
    speichert.value = true;
    try {
        emit(
            'stand',
            speichern || mitZeiten
                ? await notizSetzen(eintrag.id, speichern ? text.value : (eintrag.note ?? ''), start, ende)
                : await abschlussUeberspringen(eintrag.id),
        );
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
            <h1 class="text-base font-semibold">{{ laufend ? t.abschluss.titelLaufend : t.abschluss.titel }}</h1>
            <span v-if="anzahl > 1" class="text-[12px] text-muted-foreground">{{ mit(t.abschluss.von, { nr: nummer, anzahl }) }}</span>
        </div>

        <div class="flex items-center gap-2 rounded-lg bg-muted px-3 py-2">
            <span class="size-2.5 shrink-0 rounded-full" :style="{ background: projekt?.color ?? 'currentColor' }" />
            <div class="min-w-0 flex-1">
                <div class="truncate font-medium">{{ projekt?.name ?? `#${aktuell.project_id}` }}</div>
                <div class="truncate text-[12px] text-muted-foreground">{{ [projekt?.client_name, taetigkeit?.name].filter(Boolean).join(' · ') }}</div>
            </div>
            <span class="font-mono text-[13px] font-semibold tabular-nums">{{ laufend ? dauer(aktuell.sekunden) : mit(t.abschluss.erfasst, { dauer: dauer(sekunden) }) }}</span>
        </div>

        <div v-if="!laufend" class="flex items-center gap-4">
            <label class="flex items-center gap-2 text-[12px] text-muted-foreground">
                {{ t.abschluss.start }}
                <input
                    v-model="startWert"
                    type="time"
                    required
                    :disabled="speichert"
                    class="h-8 rounded-md border border-border bg-background px-2 text-[13px] text-foreground tabular-nums outline-none focus:border-primary"
                    @keydown.enter.prevent="weiter(true)"
                />
            </label>
            <label class="flex items-center gap-2 text-[12px] text-muted-foreground">
                {{ t.abschluss.ende }}
                <input
                    v-model="endeWert"
                    type="time"
                    required
                    :disabled="speichert"
                    class="h-8 rounded-md border border-border bg-background px-2 text-[13px] text-foreground tabular-nums outline-none focus:border-primary"
                    @keydown.enter.prevent="weiter(true)"
                />
            </label>
        </div>

        <textarea
            ref="feld"
            v-model="text"
            rows="5"
            maxlength="2000"
            :placeholder="t.abschluss.platzhalter"
            :aria-label="laufend ? t.abschluss.titelLaufend : t.abschluss.titel"
            :disabled="speichert"
            class="w-full resize-none rounded-lg border border-border bg-background px-3 py-2 outline-none focus:border-primary"
            @keydown="taste"
        />
        <p class="-mt-2 text-[11px] text-muted-foreground">{{ t.abschluss.hinweis }}</p>
        <Vorschlaege v-model="text" class="-mt-1" :projekt="aktuell.project_id" :taetigkeit="aktuell.activity_id" />

        <button type="button" class="h-10 rounded-lg bg-primary font-medium text-primary-foreground disabled:opacity-50" :disabled="speichert" @click="weiter(true)">
            {{ t.abschluss.speichern }}
        </button>
        <button type="button" class="h-10 rounded-lg border border-border font-medium hover:bg-muted disabled:opacity-50" :disabled="speichert" @click="weiter(false)">
            {{ laufend ? t.haupt.abbrechen : t.abschluss.ohne }}
        </button>
        <button v-if="!laufend" type="button" class="mt-auto self-center text-[12px] text-muted-foreground underline-offset-2 hover:underline" @click="emit('nichtMehrFragen')">
            {{ t.abschluss.nichtMehr }}
        </button>
    </main>
</template>
