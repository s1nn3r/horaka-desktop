<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { alsFehler, startSetzen } from '../befehle';
import { t } from '../i18n';
import type { Fehler, Stand } from '../typen';
import { letzteUhrzeit, uhrzeit } from '../zeit';

// Start eines laufenden Timers nachträglich korrigieren (Kontextmenü):
// Uhrzeit, gemeint ist ihr letztes Vorkommen bis jetzt. Enter speichert,
// Esc bricht ab. Der Server rundet nach den Regeln des Workspace.
const props = defineProps<{ id: number; start: string }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler]; zu: [] }>();

const offen = ref(false);
const wert = ref('');
const speichert = ref(false);
const feld = ref<HTMLInputElement | null>(null);

function oeffnen() {
    wert.value = uhrzeit(props.start);
    offen.value = true;
    void nextTick(() => feld.value?.focus());
}

function schliessen() {
    offen.value = false;
    emit('zu');
}

async function speichern() {
    const neu = letzteUhrzeit(wert.value, new Date());
    if (speichert.value || !neu) {
        return;
    }
    if (wert.value === uhrzeit(props.start)) {
        schliessen();
        return;
    }
    speichert.value = true;
    try {
        emit('stand', await startSetzen(props.id, neu.toISOString()));
        schliessen();
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        speichert.value = false;
    }
}

function taste(e: KeyboardEvent) {
    if (e.key === 'Enter') {
        e.preventDefault();
        void speichern();
    } else if (e.key === 'Escape') {
        // Nicht bis zum Fenster durchreichen, das sich sonst schließt.
        e.stopPropagation();
        schliessen();
    }
}

defineExpose({ oeffnen });
</script>

<template>
    <div v-if="offen" class="mt-1 flex flex-wrap items-center gap-2">
        <label class="flex items-center gap-2 text-[12px] text-muted-foreground">
            {{ t.haupt.start }}
            <input
                ref="feld"
                v-model="wert"
                type="time"
                required
                :disabled="speichert"
                class="h-7 rounded-md border border-border bg-background px-2 text-[13px] text-foreground tabular-nums outline-none focus:border-primary"
                @keydown="taste"
            />
        </label>
        <div class="ml-auto flex items-center gap-2">
            <button type="button" class="h-7 rounded-md px-2 text-[12px] font-medium text-muted-foreground hover:bg-muted" @click="schliessen">
                {{ t.haupt.abbrechen }}
            </button>
            <button type="button" class="h-7 rounded-md bg-primary px-2.5 text-[12px] font-medium text-primary-foreground" :disabled="speichert" @click="speichern">
                {{ t.haupt.speichern }}
            </button>
        </div>
    </div>
</template>
