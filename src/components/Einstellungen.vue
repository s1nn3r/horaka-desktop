<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import { onMounted, ref } from 'vue';
import { alsFehler, ortGemerkt, ortZuruecksetzen, trennen, updateSuchen } from '../befehle';
import { mit, SPRACHEN, t } from '../i18n';
import { anzeigen } from '../kuerzel';
import type { Auswahl, Einstellungen, Fehler, Kuerzel, Stand, Thema } from '../typen';
import KuerzelListe from './KuerzelListe.vue';

// Einstellungen der App: Darstellung, Sprache, Kompaktansicht, Notiz beim Stoppen, Tastenkürzel,
// Autostart, Fensterposition, Verbindung. Gespeichert wird in App.vue
// (Ereignis `aendern`), die Antwort sagt, welche Kürzel belegt sind.
const props = defineProps<{ einstellungen: Einstellungen; belegt: string[]; auswahl: Auswahl | null; update: string | null }>();
const emit = defineEmits<{
    zurueck: [];
    getrennt: [stand: Stand];
    fehler: [fehler: Fehler];
    aendern: [neu: Einstellungen];
    stand: [stand: Stand];
    aktualisieren: [];
    /** Deckkraft in Prozent, solange der Regler gezogen wird, sonst null. */
    deckkraftVorschau: [prozent: number | null];
}>();

// Regler: beim Ziehen zeigt das Fenster die Deckkraft, beim Loslassen wird
// gespeichert und das Fenster ist wieder normal.
const deckkraft = ref(props.einstellungen.kompakt_deckkraft);
function deckkraftZiehen(e: Event) {
    deckkraft.value = Number((e.target as HTMLInputElement).value);
    emit('deckkraftVorschau', deckkraft.value);
}
function deckkraftLoslassen() {
    emit('deckkraftVorschau', null);
    if (deckkraft.value !== props.einstellungen.kompakt_deckkraft) {
        aendern({ kompakt_deckkraft: deckkraft.value });
    }
}

const THEMEN: Thema[] = ['hell', 'dunkel', 'system'];

const autostart = ref(false);
const version = ref('');
const gemerkt = ref(false);
const sucht = ref(false);
const geprueft = ref(false);

onMounted(async () => {
    version.value = await getVersion();
    gemerkt.value = await ortGemerkt();
    try {
        autostart.value = await isEnabled();
    } catch (e) {
        emit('fehler', alsFehler(e));
    }
});

function aendern(teil: Partial<Einstellungen>) {
    emit('aendern', { ...props.einstellungen, ...teil });
}

async function autostartSetzen(an: boolean) {
    try {
        await (an ? enable() : disable());
        autostart.value = await isEnabled();
    } catch (e) {
        emit('fehler', alsFehler(e));
    }
}

async function jetztSuchen() {
    sucht.value = true;
    try {
        emit('stand', await updateSuchen());
        geprueft.value = true;
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        sucht.value = false;
    }
}

async function ortLoesen() {
    await ortZuruecksetzen();
    gemerkt.value = false;
}

async function verbindungTrennen() {
    emit('getrennt', await trennen());
}
</script>

<template>
    <header data-tauri-drag-region class="flex cursor-default items-center gap-2 border-b border-border px-2 py-2.5">
        <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.einstellungen.zurueck" @click="emit('zurueck')">
            <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true">
                <path d="M10 3L5 8l5 5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>
        <h1 data-tauri-drag-region class="flex-1 font-semibold">{{ t.einstellungen.titel }}</h1>
    </header>

    <main class="flex flex-1 flex-col gap-6 overflow-y-auto p-4">
        <section>
            <h2 class="mb-1.5 font-medium">{{ t.einstellungen.thema }}</h2>
            <!-- So breit wie der Inhalt, Symbole wie in Horaka (lucide sun, moon, monitor). -->
            <div class="inline-flex max-w-full gap-1 rounded-lg bg-muted p-1" role="radiogroup" :aria-label="t.einstellungen.thema">
                <button
                    v-for="thema in THEMEN"
                    :key="thema"
                    type="button"
                    role="radio"
                    :aria-checked="einstellungen.thema === thema"
                    class="inline-flex h-8 items-center gap-1.5 rounded-md px-2.5 text-[12px] font-medium whitespace-nowrap"
                    :class="einstellungen.thema === thema ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
                    @click="aendern({ thema })"
                >
                    <svg viewBox="0 0 24 24" class="size-3.5 shrink-0" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                        <template v-if="thema === 'hell'">
                            <circle cx="12" cy="12" r="4" />
                            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41" />
                        </template>
                        <path v-else-if="thema === 'dunkel'" d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
                        <template v-else>
                            <rect width="20" height="14" x="2" y="3" rx="2" />
                            <path d="M8 21h8M12 17v4" />
                        </template>
                    </svg>
                    {{ t.einstellungen.themen[thema] }}
                </button>
            </div>
        </section>

        <section>
            <label for="sprache" class="mb-1.5 block font-medium">{{ t.einstellungen.sprache }}</label>
            <select
                id="sprache"
                class="h-9 w-full rounded-lg border border-border bg-muted px-2 outline-none focus:border-primary"
                :value="einstellungen.sprache"
                @change="aendern({ sprache: ($event.target as HTMLSelectElement).value })"
            >
                <option value="system">{{ t.einstellungen.spracheSystem }}</option>
                <option v-for="(name, code) in SPRACHEN" :key="code" :value="code" :lang="code">{{ name }}</option>
            </select>
            <p class="mt-1 text-[12px] text-muted-foreground">{{ t.einstellungen.spracheText }}</p>
        </section>

        <label class="flex items-start gap-3">
            <input
                type="checkbox"
                class="mt-0.5 size-4 accent-[var(--primary)]"
                :checked="einstellungen.kompakt"
                @change="aendern({ kompakt: ($event.target as HTMLInputElement).checked })"
            />
            <span>
                <span class="block font-medium">{{ t.einstellungen.kompakt }}</span>
                <span class="block text-[12px] text-muted-foreground">{{ t.einstellungen.kompaktText }}</span>
            </span>
        </label>

        <!-- Auch ohne Kompaktansicht einstellbar (man kommt nur über die volle Ansicht hierher), wirkt nur dort. -->
        <label class="-mt-3 ml-7 flex items-start gap-3">
            <input
                type="checkbox"
                class="mt-0.5 size-4 accent-[var(--primary)]"
                :checked="einstellungen.kompakt_durchsichtig"
                @change="aendern({ kompakt_durchsichtig: ($event.target as HTMLInputElement).checked })"
            />
            <span>
                <span class="block">{{ t.einstellungen.durchsichtig }}</span>
                <span class="block text-[12px] text-muted-foreground">{{ t.einstellungen.durchsichtigText }}</span>
            </span>
        </label>

        <div v-if="einstellungen.kompakt_durchsichtig" class="-mt-3 ml-14">
            <label for="deckkraft" class="flex items-center justify-between text-[12px]">
                <span>{{ t.einstellungen.deckkraft }}</span>
                <span class="font-mono tabular-nums">{{ deckkraft }} %</span>
            </label>
            <input
                id="deckkraft"
                type="range"
                min="10"
                max="90"
                step="5"
                class="mt-1 w-full accent-[var(--primary)]"
                :value="deckkraft"
                @input="deckkraftZiehen"
                @change="deckkraftLoslassen"
                @pointerup="deckkraftLoslassen"
                @blur="emit('deckkraftVorschau', null)"
            />
        </div>

        <section>
            <h2 class="mb-1.5 font-medium">{{ t.einstellungen.notizen }}</h2>
            <label class="flex items-start gap-3">
                <input
                    type="checkbox"
                    class="mt-0.5 size-4 accent-[var(--primary)]"
                    :checked="einstellungen.notiz_beim_stoppen"
                    @change="aendern({ notiz_beim_stoppen: ($event.target as HTMLInputElement).checked })"
                />
                <span>
                    <span class="block">{{ t.einstellungen.notizBeimStoppen }}</span>
                    <span class="block text-[12px] text-muted-foreground">{{ t.einstellungen.notizBeimStoppenText }}</span>
                </span>
            </label>
        </section>

        <section>
            <h2 class="font-medium">{{ t.einstellungen.kuerzelTitel }}</h2>
            <p class="mb-2 text-[12px] text-muted-foreground">{{ t.einstellungen.kuerzelText }}</p>
            <p v-if="belegt.length > 0" role="alert" class="mb-2 text-[12px] text-destructive">
                {{ mit(t.einstellungen.kuerzelBelegt, { liste: belegt.map(anzeigen).join(', ') }) }}
            </p>
            <KuerzelListe :kuerzel="einstellungen.kuerzel" :auswahl="auswahl" :belegt="belegt" @aendern="(kuerzel: Kuerzel[]) => aendern({ kuerzel })" />
        </section>

        <section>
            <h2 class="mb-1.5 font-medium">{{ t.update.titel }}</h2>
            <label class="flex items-start gap-3">
                <input
                    type="checkbox"
                    class="mt-0.5 size-4 accent-[var(--primary)]"
                    :checked="einstellungen.updates_suchen"
                    @change="aendern({ updates_suchen: ($event.target as HTMLInputElement).checked })"
                />
                <span>
                    <span class="block">{{ t.update.suchen }}</span>
                    <span class="block text-[12px] text-muted-foreground">{{ t.update.suchenText }}</span>
                </span>
            </label>
            <div class="mt-2 flex flex-wrap items-center gap-2">
                <button v-if="update" type="button" class="h-9 rounded-lg bg-primary px-3 font-medium text-primary-foreground" @click="emit('aktualisieren')">
                    {{ t.update.installieren }}
                </button>
                <button v-else type="button" class="h-9 rounded-lg border border-border px-3 font-medium hover:bg-muted disabled:opacity-50" :disabled="sucht" @click="jetztSuchen">
                    {{ sucht ? t.update.sucht : t.update.jetzt }}
                </button>
                <span v-if="update" class="text-[12px] text-muted-foreground">{{ mit(t.update.da, { version: update }) }}</span>
                <span v-else-if="geprueft" class="text-[12px] text-muted-foreground">{{ t.update.aktuell }}</span>
            </div>
        </section>

        <label class="flex items-start gap-3">
            <input
                type="checkbox"
                class="mt-0.5 size-4 accent-[var(--primary)]"
                :checked="autostart"
                @change="autostartSetzen(($event.target as HTMLInputElement).checked)"
            />
            <span>
                <span class="block font-medium">{{ t.einstellungen.autostart }}</span>
                <span class="block text-[12px] text-muted-foreground">{{ t.einstellungen.autostartText }}</span>
            </span>
        </label>

        <section>
            <h2 class="font-medium">{{ t.einstellungen.ort }}</h2>
            <p class="text-[12px] text-muted-foreground">{{ gemerkt ? t.einstellungen.ortGemerkt : t.einstellungen.ortSymbol }}</p>
            <button v-if="gemerkt" type="button" class="mt-1.5 h-9 rounded-lg border border-border px-3 font-medium hover:bg-muted" @click="ortLoesen">
                {{ t.einstellungen.ortZuruecksetzen }}
            </button>
        </section>

        <section>
            <button type="button" class="h-9 rounded-lg border border-border px-3 font-medium text-destructive hover:bg-muted" @click="verbindungTrennen">
                {{ t.einstellungen.trennen }}
            </button>
            <p class="mt-1.5 text-[12px] text-muted-foreground">{{ t.einstellungen.trennenText }}</p>
        </section>

        <p class="mt-auto text-[12px] text-muted-foreground">{{ t.einstellungen.version }} {{ version }}</p>
    </main>
</template>
