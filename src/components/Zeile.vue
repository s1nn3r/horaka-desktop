<script setup lang="ts">
import { computed } from 'vue';
import { t } from '../i18n';
import type { Aktion, Projekt, Taetigkeit } from '../typen';

// Eine buchbare Kombination: Farbe, „Kunde · Projekt“, Tätigkeit, rechts
// Start (oder Stopp, wenn sie läuft); mit parallelen Timern und einem
// laufenden Timer zusätzlich „Hierhin wechseln“.
const props = defineProps<{
    projekt: Projekt;
    taetigkeit: Taetigkeit | null;
    laeuft: boolean;
    parallel: boolean;
    andereLaufen: boolean;
    beschaeftigt: boolean;
    aufklappbar?: boolean;
    offen?: boolean;
}>();

const emit = defineEmits<{ aktion: [aktion: Aktion]; umschalten: [] }>();

const unterzeile = computed(() => {
    const teile = [props.projekt.client_name];
    if (props.taetigkeit) {
        teile.push(props.taetigkeit.name);
    }
    if (props.projekt.status === 'paused') {
        teile.push(t.haupt.pausiert);
    }
    return teile.join(' · ');
});
</script>

<template>
    <div class="group flex min-h-11 items-center gap-2 rounded-lg px-2 hover:bg-muted">
        <span class="size-2.5 shrink-0 rounded-full" :style="{ background: projekt.color }" />

        <button
            type="button"
            class="flex min-w-0 flex-1 flex-col items-start py-1.5 text-left"
            :disabled="beschaeftigt"
            :title="laeuft ? t.haupt.stoppen : t.haupt.starten"
            @click="emit('aktion', laeuft ? 'stop' : 'start')"
        >
            <span class="w-full truncate font-medium">{{ projekt.name }}</span>
            <span class="w-full truncate text-[12px] text-muted-foreground">{{ unterzeile }}</span>
        </button>

        <button
            v-if="aufklappbar && projekt.activities.length > 0"
            type="button"
            class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-background"
            :aria-expanded="offen"
            :title="t.haupt.taetigkeiten"
            @click="emit('umschalten')"
        >
            <svg viewBox="0 0 16 16" class="size-4 transition-transform" :class="{ 'rotate-90': offen }" aria-hidden="true">
                <path d="M6 3l5 5-5 5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>

        <button
            v-if="!laeuft && parallel && andereLaufen"
            type="button"
            class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-background hover:text-foreground"
            :disabled="beschaeftigt"
            :title="t.haupt.wechseln"
            @click="emit('aktion', 'switch')"
        >
            <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true">
                <path d="M3 5h9l-2.5-2.5M13 11H4l2.5 2.5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>

        <button
            type="button"
            class="flex size-8 shrink-0 items-center justify-center rounded-full"
            :class="laeuft ? 'bg-destructive text-destructive-foreground' : 'bg-primary-tint text-primary group-hover:bg-primary group-hover:text-primary-foreground'"
            :disabled="beschaeftigt"
            :title="laeuft ? t.haupt.stoppen : parallel && andereLaufen ? t.haupt.zusaetzlich : t.haupt.starten"
            @click="emit('aktion', laeuft ? 'stop' : 'start')"
        >
            <svg v-if="laeuft" viewBox="0 0 16 16" class="size-3.5" aria-hidden="true"><rect x="3.5" y="3.5" width="9" height="9" rx="1.5" fill="currentColor" /></svg>
            <svg v-else viewBox="0 0 16 16" class="ml-0.5 size-3.5" aria-hidden="true"><path d="M4.5 2.8v10.4L13 8z" fill="currentColor" /></svg>
        </button>
    </div>
</template>
