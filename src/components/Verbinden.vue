<script setup lang="ts">
import { ref } from 'vue';
import { alsFehler, anmelden, anmeldungAbbrechen, appOeffnen, verbergen, verbinden } from '../befehle';
import { t } from '../i18n';
import type { Fehler, Stand } from '../typen';
import Zeichen from './Zeichen.vue';

const emit = defineEmits<{ verbunden: [stand: Stand]; fehler: [fehler: Fehler] }>();

const wartet = ref(false);
const mitToken = ref(false);
const token = ref('');
const laeuft = ref(false);

// Hauptweg: OAuth im Browser, die App wartet auf den Rücksprung.
async function imBrowser() {
    wartet.value = true;
    try {
        emit('verbunden', await anmelden());
    } catch (e) {
        const fehler = alsFehler(e);
        if (fehler.code !== 'oauth_cancelled') {
            emit('fehler', fehler);
        }
    } finally {
        wartet.value = false;
    }
}

async function mitGeraetetoken() {
    if (laeuft.value) {
        return;
    }
    laeuft.value = true;
    try {
        const stand = await verbinden(token.value);
        token.value = '';
        emit('verbunden', stand);
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        laeuft.value = false;
    }
}
</script>

<template>
    <div class="flex flex-1 flex-col gap-4 overflow-y-auto p-5">
        <div data-tauri-drag-region class="-mx-5 -mt-5 flex cursor-default items-center gap-3 px-5 pt-5 pb-1">
            <Zeichen class="pointer-events-none size-9" />
            <h1 data-tauri-drag-region class="flex-1 text-base font-semibold">{{ t.verbinden.titel }}</h1>
            <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.haupt.schliessen" @click="verbergen()">
                <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
            </button>
        </div>

        <template v-if="wartet">
            <div class="flex items-center gap-3 rounded-lg bg-primary-tint p-3">
                <span class="size-4 shrink-0 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                <span class="font-medium">{{ t.verbinden.warten }}</span>
            </div>
            <p class="text-muted-foreground">{{ t.verbinden.wartenText }}</p>
            <div class="rounded-lg border border-primary/40 bg-primary-tint p-3 text-[13px]">
                <div class="mb-0.5 font-semibold text-primary">{{ t.verbinden.rechteTitel }}</div>
                {{ t.verbinden.rechte }}
            </div>
            <button type="button" class="h-10 rounded-lg border border-border font-medium hover:bg-muted" @click="anmeldungAbbrechen()">
                {{ t.verbinden.abbrechen }}
            </button>
        </template>

        <template v-else>
            <p class="text-muted-foreground">{{ t.verbinden.text }}</p>
            <div class="rounded-lg border border-primary/40 bg-primary-tint p-3 text-[13px]">
                <div class="mb-0.5 font-semibold text-primary">{{ t.verbinden.rechteTitel }}</div>
                {{ t.verbinden.rechte }}
            </div>
            <button type="button" class="h-10 rounded-lg bg-primary font-medium text-primary-foreground" autofocus @click="imBrowser">
                {{ t.verbinden.anmelden }}
            </button>

            <button
                type="button"
                class="self-start text-[13px] font-medium text-primary underline-offset-2 hover:underline"
                :aria-expanded="mitToken"
                @click="mitToken = !mitToken"
            >
                {{ t.verbinden.alternativ }}
            </button>

            <form v-if="mitToken" class="flex flex-col gap-3 border-t border-border pt-4" @submit.prevent="mitGeraetetoken">
                <p class="text-[13px] text-muted-foreground">{{ t.verbinden.tokenText }}</p>
                <button
                    type="button"
                    class="self-start text-[13px] font-medium text-primary underline-offset-2 hover:underline"
                    @click="appOeffnen('einstellungen/api')"
                >
                    {{ t.verbinden.einstellungen }}
                </button>
                <label class="flex flex-col gap-1.5">
                    <span class="text-[13px] font-medium">{{ t.verbinden.token }}</span>
                    <input
                        v-model="token"
                        type="password"
                        autocomplete="off"
                        spellcheck="false"
                        :placeholder="t.verbinden.platzhalter"
                        class="h-10 rounded-lg border border-border bg-background px-3 font-mono text-[13px] outline-none focus:border-primary"
                    />
                </label>
                <button
                    type="submit"
                    :disabled="laeuft || token.trim() === ''"
                    class="h-10 rounded-lg border border-border font-medium hover:bg-muted disabled:opacity-50"
                >
                    {{ laeuft ? t.verbinden.laeuft : t.verbinden.knopf }}
                </button>
            </form>
        </template>

        <p class="mt-auto text-[12px] text-muted-foreground">{{ t.verbinden.hinweis }}</p>
    </div>
</template>
