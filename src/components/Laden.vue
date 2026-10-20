<script setup lang="ts">
import { t } from '../i18n';
import Kopf from './Kopf.vue';

// Verbunden, aber die Daten fehlen noch (Start ohne Netz): kein Rückfall
// auf „Verbinden“, sondern warten; der Abgleich versucht es alle 5 Sekunden.
defineProps<{ offline: boolean }>();
const emit = defineEmits<{ erneut: []; einstellungen: [] }>();
</script>

<template>
    <Kopf @einstellungen="emit('einstellungen')" />
    <main class="flex flex-1 flex-col items-center justify-center gap-4 p-6 text-center">
        <span class="size-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        <p class="font-medium">{{ t.laden.titel }}</p>
        <p v-if="offline" class="text-muted-foreground">{{ t.laden.offline }}</p>
        <button v-if="offline" type="button" class="h-9 rounded-lg border border-border px-3 font-medium hover:bg-muted" @click="emit('erneut')">
            {{ t.laden.erneut }}
        </button>
    </main>
</template>
