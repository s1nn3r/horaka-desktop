<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { notizVorschlaege } from '../befehle';
import { t } from '../i18n';

// Vorschläge unter dem Notizfeld wie die Antwortvorschläge in Teams: die
// zuletzt genutzten Zeilen aus eigenen Notizen des Projekts (mit Tätigkeit
// nur dieser), ohne die, die schon in der Notiz stehen. Die Pille kürzt,
// übernommen wird der ganze Text als neue Zeile unten. mousedown.prevent
// hält den Fokus im Feld.
const props = defineProps<{ projekt: number; taetigkeit: number | null }>();
const notiz = defineModel<string>({ required: true });

const vorschlaege = ref<string[]>([]);
watch(
    () => [props.projekt, props.taetigkeit] as const,
    async ([projekt, taetigkeit]) => {
        vorschlaege.value = [];
        try {
            vorschlaege.value = await notizVorschlaege(projekt, taetigkeit);
        } catch {
            // Ohne Netz keine Vorschläge.
        }
    },
    { immediate: true },
);

const offen = computed(() => {
    const da = new Set(notiz.value.split(/\r\n|\r|\n/).map((z) => z.trim().toLowerCase()));
    return vorschlaege.value.filter((v) => !da.has(v.toLowerCase()));
});

function uebernehmen(zeile: string) {
    const bisher = notiz.value.trimEnd();
    notiz.value = bisher === '' ? zeile : `${bisher}\n${zeile}`;
}
</script>

<template>
    <div v-if="offen.length" class="mt-1.5 flex flex-wrap gap-1" role="group" :aria-label="t.haupt.vorschlaege">
        <button
            v-for="zeile in offen"
            :key="zeile"
            type="button"
            :title="zeile"
            class="flex h-6 max-w-40 items-center rounded-full border border-border bg-background px-2 text-[11px] hover:bg-muted"
            @mousedown.prevent
            @click="uebernehmen(zeile)"
        >
            <span class="truncate">{{ zeile }}</span>
        </button>
    </div>
</template>
