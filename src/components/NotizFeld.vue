<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { alsFehler, notizSetzen } from '../befehle';
import { t } from '../i18n';
import type { Fehler, Stand } from '../typen';

// Notiz am laufenden Timer: zeigt sie, ein Klick öffnet das Feld. Enter
// speichert, Umschalt+Enter macht eine neue Zeile, Esc bricht ab.
const props = defineProps<{ id: number; notiz: string | null }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler] }>();

const offen = ref(false);
const text = ref('');
const speichert = ref(false);
const feld = ref<HTMLTextAreaElement | null>(null);

function oeffnen() {
    text.value = props.notiz ?? '';
    offen.value = true;
    void nextTick(() => {
        feld.value?.focus();
        feld.value?.setSelectionRange(text.value.length, text.value.length);
    });
}

async function speichern() {
    if (speichert.value) {
        return;
    }
    if (text.value.trim() === (props.notiz ?? '').trim()) {
        offen.value = false;
        return;
    }
    speichert.value = true;
    try {
        emit('stand', await notizSetzen(props.id, text.value));
        offen.value = false;
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        speichert.value = false;
    }
}

function taste(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        void speichern();
    } else if (e.key === 'Escape') {
        // Nicht bis zum Fenster durchreichen, das sich sonst schließt.
        e.stopPropagation();
        offen.value = false;
    }
}

defineExpose({ oeffnen });
</script>

<template>
    <div class="mt-1">
        <div v-if="offen">
            <textarea
                ref="feld"
                v-model="text"
                rows="3"
                maxlength="2000"
                :aria-label="t.haupt.notiz"
                :placeholder="t.haupt.notizPlatzhalter"
                :disabled="speichert"
                class="w-full resize-none rounded-lg border border-border bg-background px-2.5 py-1.5 text-[13px] outline-none focus:border-primary"
                @keydown="taste"
            />
            <div class="mt-1 flex items-center gap-2">
                <span class="flex-1 text-[11px] text-muted-foreground">{{ t.haupt.notizHinweis }}</span>
                <button type="button" class="h-7 rounded-md px-2 text-[12px] font-medium text-muted-foreground hover:bg-muted" @click="offen = false">
                    {{ t.haupt.abbrechen }}
                </button>
                <button type="button" class="h-7 rounded-md bg-primary px-2.5 text-[12px] font-medium text-primary-foreground" :disabled="speichert" @click="speichern">
                    {{ t.haupt.speichern }}
                </button>
            </div>
        </div>
        <button
            v-else
            type="button"
            class="line-clamp-2 w-full rounded-md px-1 -mx-1 text-left text-[12px] whitespace-pre-line hover:bg-muted"
            :class="notiz ? 'text-foreground' : 'text-muted-foreground italic'"
            :title="t.haupt.notiz"
            @click="oeffnen"
        >
            {{ notiz || t.haupt.notizPlatzhalter }}
        </button>
    </div>
</template>
