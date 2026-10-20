<script setup lang="ts">
import { appOeffnen, verbergen } from '../befehle';
import { t } from '../i18n';
import Zeichen from './Zeichen.vue';

// Kopfzeile des Fensters. Sie ist zugleich der Griff zum Verschieben
// (data-tauri-drag-region; nur das Element selbst zieht, nicht die Knöpfe).
const emit = defineEmits<{ einstellungen: []; kompakt: [] }>();
</script>

<template>
    <header data-tauri-drag-region class="flex cursor-default items-center gap-2 border-b border-border px-3 py-2.5">
        <Zeichen class="pointer-events-none size-6" />
        <span data-tauri-drag-region class="flex-1 font-semibold">Horaka</span>
        <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.haupt.oeffnen" @click="appOeffnen()">
            <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true">
                <path d="M9 3h4v4M13 3L7.5 8.5M11 9.5V13H3V5h3.5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>
        <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.haupt.kompakt" @click="emit('kompakt')">
            <!-- Verkleinern (Form wie lucide „minimize-2“). -->
            <svg viewBox="0 0 24 24" class="size-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M4 14h6v6M20 10h-6V4M14 10l7-7M3 21l7-7" />
            </svg>
        </button>
        <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.haupt.einstellungen" @click="emit('einstellungen')">
            <!-- Zahnrad (Form wie lucide „settings“). -->
            <svg viewBox="0 0 24 24" class="size-4" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path
                    d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
                />
                <circle cx="12" cy="12" r="3" />
            </svg>
        </button>
        <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.haupt.schliessen" @click="verbergen()">
            <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
        </button>
    </header>
</template>
