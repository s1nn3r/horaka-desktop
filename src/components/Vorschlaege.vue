<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { notizVorschlaege } from '../befehle';
import { t } from '../i18n';

// Vorschläge unter dem Notizfeld wie die Antwortvorschläge in Teams:
// zuletzt und häufig genutzte Zeilen aus eigenen Notizen des Projekts (mit
// Tätigkeit nur dieser), ohne die, die schon in der Notiz stehen. Die Pille
// kürzt (zwei lange passen nebeneinander), übernommen wird der ganze Text als
// neue Zeile unten. mousedown.prevent hält den Fokus im Feld.
const props = defineProps<{ projekt: number; taetigkeit: number | null }>();
const notiz = defineModel<string>({ required: true });

// Mit der Notiz im Feld: was schon drinsteht, ersetzt der Server durch die
// nächsten. Geladen beim Öffnen und nach jedem Klick auf eine Pille.
const vorschlaege = ref<string[]>([]);
let anfrage = 0;
// `text` direkt nach einem Klick: das Modell zeigt den neuen Wert erst nach dem Rendern.
async function laden(text = notiz.value) {
    const nr = ++anfrage;
    try {
        const neu = await notizVorschlaege(props.projekt, props.taetigkeit, text);
        if (nr === anfrage) {
            vorschlaege.value = neu;
        }
    } catch {
        // Ohne Netz bleiben die bisherigen.
    }
}
watch(
    () => [props.projekt, props.taetigkeit],
    () => {
        vorschlaege.value = [];
        void laden();
    },
    { immediate: true },
);

const offen = computed(() => {
    const da = new Set(notiz.value.split(/\r\n|\r|\n/).map((z) => z.trim().toLowerCase()));
    return vorschlaege.value.filter((v) => !da.has(v.toLowerCase()));
});

function uebernehmen(zeile: string) {
    const bisher = notiz.value.trimEnd();
    const neu = bisher === '' ? zeile : `${bisher}\n${zeile}`;
    notiz.value = neu;
    void laden(neu);
}
</script>

<template>
    <div v-if="offen.length" class="flex flex-wrap gap-1" role="group" :aria-label="t.haupt.vorschlaege">
        <button
            v-for="zeile in offen"
            :key="zeile"
            type="button"
            :title="zeile"
            class="flex h-6 max-w-[calc(50%-2px)] items-center rounded-full border border-primary/40 bg-primary-tint px-2 text-[11px] font-medium text-primary hover:border-primary"
            @mousedown.prevent
            @click="uebernehmen(zeile)"
        >
            <span class="truncate">{{ zeile }}</span>
        </button>
    </div>
</template>
