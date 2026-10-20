<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app';
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart';
import { onMounted, ref } from 'vue';
import { alsFehler, trennen, updateSuchen } from '../befehle';
import { mit, SPRACHEN, t } from '../i18n';
import { anzeigen } from '../kuerzel';
import type { Auswahl, Einstellungen, Fehler, Kuerzel, Stand, Thema } from '../typen';
import KuerzelListe from './KuerzelListe.vue';
import SchalterZeile from './SchalterZeile.vue';

// Einstellungen der App in drei Registern: Allgemein (Darstellung, Sprache,
// Kompaktansicht, Notiz beim Stoppen), Tastenkürzel, System (Autostart,
// Updates, Verbindung). Gespeichert wird in App.vue
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
const REITER = ['allgemein', 'kuerzel', 'system'] as const;
const reiter = ref<(typeof REITER)[number]>('allgemein');
const inhalt = ref<HTMLElement | null>(null);
function reiterWaehlen(r: (typeof REITER)[number]) {
    reiter.value = r;
    inhalt.value?.scrollTo(0, 0);
}
const reiterTitel = { allgemein: t.einstellungen.allgemein, kuerzel: t.einstellungen.kuerzelReiter, system: t.einstellungen.system };

const autostart = ref(false);
const version = ref('');
const sucht = ref(false);
const geprueft = ref(false);

onMounted(async () => {
    version.value = await getVersion();
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

async function verbindungTrennen() {
    emit('getrennt', await trennen());
}
</script>

<template>
    <header data-tauri-drag-region class="flex cursor-default items-center gap-2 px-2 pt-2.5 pb-2">
        <button type="button" class="flex size-8 items-center justify-center rounded-md text-muted-foreground hover:bg-muted" :title="t.einstellungen.zurueck" @click="emit('zurueck')">
            <svg viewBox="0 0 16 16" class="size-4" aria-hidden="true">
                <path d="M10 3L5 8l5 5" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
        </button>
        <h1 data-tauri-drag-region class="flex-1 font-semibold">{{ t.einstellungen.titel }}</h1>
    </header>

    <div class="border-b border-border px-3 pb-2.5">
        <div class="grid grid-cols-3 gap-1 rounded-lg bg-muted p-1" role="tablist">
            <button
                v-for="r in REITER"
                :key="r"
                type="button"
                role="tab"
                :aria-selected="reiter === r"
                class="h-7 truncate rounded-md px-1.5 text-[12px] font-medium"
                :class="reiter === r ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
                @click="reiterWaehlen(r)"
            >
                {{ reiterTitel[r] }}
            </button>
        </div>
    </div>

    <main ref="inhalt" class="flex flex-1 flex-col gap-5 overflow-y-auto p-3">
        <template v-if="reiter === 'allgemein'">
            <section>
                <h2 class="mb-1.5 px-1 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">{{ t.einstellungen.thema }}</h2>
                <div class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-muted/40">
                    <div class="px-3 py-2.5">
                        <!-- Symbole wie in Horaka (lucide sun, moon, monitor). -->
                        <div class="flex gap-1 rounded-lg bg-muted p-1" role="radiogroup" :aria-label="t.einstellungen.thema">
                            <button
                                v-for="thema in THEMEN"
                                :key="thema"
                                type="button"
                                role="radio"
                                :aria-checked="einstellungen.thema === thema"
                                class="inline-flex h-7 min-w-0 flex-auto items-center justify-center gap-1.5 rounded-md px-2 text-[12px] font-medium"
                                :class="einstellungen.thema === thema ? 'bg-background shadow-sm' : 'text-muted-foreground hover:text-foreground'"
                                :title="t.einstellungen.themen[thema]"
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
                                <span class="truncate">{{ t.einstellungen.themen[thema] }}</span>
                            </button>
                        </div>
                    </div>
                    <div class="flex items-center gap-3 px-3 py-2.5">
                        <label for="sprache" class="min-w-0 flex-1">
                            <span class="block text-[13px] font-medium">{{ t.einstellungen.sprache }}</span>
                            <span class="block text-[12px] text-muted-foreground">{{ t.einstellungen.spracheText }}</span>
                        </label>
                        <select
                            id="sprache"
                            class="h-8 max-w-[45%] rounded-md border border-border bg-background px-2 text-[13px] outline-none focus:border-primary"
                            :value="einstellungen.sprache"
                            @change="aendern({ sprache: ($event.target as HTMLSelectElement).value })"
                        >
                            <option value="system">{{ t.einstellungen.spracheSystem }}</option>
                            <option v-for="(name, code) in SPRACHEN" :key="code" :value="code" :lang="code">{{ name }}</option>
                        </select>
                    </div>
                </div>
            </section>

            <section>
                <h2 class="mb-1.5 px-1 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">{{ t.einstellungen.kompakt }}</h2>
                <div class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-muted/40">
                    <SchalterZeile :titel="t.einstellungen.kompakt" :text="t.einstellungen.kompaktText" :an="einstellungen.kompakt" @aendern="(an) => aendern({ kompakt: an })" />
                    <!-- Auch ohne Kompaktansicht einstellbar (man kommt nur über die volle Ansicht hierher), wirkt nur dort. -->
                    <SchalterZeile
                        :titel="t.einstellungen.durchsichtig"
                        :text="t.einstellungen.durchsichtigText"
                        :an="einstellungen.kompakt_durchsichtig"
                        @aendern="(an) => aendern({ kompakt_durchsichtig: an })"
                    />
                    <div v-if="einstellungen.kompakt_durchsichtig" class="px-3 py-2.5">
                        <label for="deckkraft" class="flex items-center justify-between text-[13px] font-medium">
                            <span>{{ t.einstellungen.deckkraft }}</span>
                            <span class="font-mono text-[12px] text-muted-foreground tabular-nums">{{ deckkraft }} %</span>
                        </label>
                        <input
                            id="deckkraft"
                            type="range"
                            min="10"
                            max="90"
                            step="5"
                            class="mt-1.5 w-full accent-[var(--primary)]"
                            :value="deckkraft"
                            @input="deckkraftZiehen"
                            @change="deckkraftLoslassen"
                            @pointerup="deckkraftLoslassen"
                            @blur="emit('deckkraftVorschau', null)"
                        />
                    </div>
                </div>
            </section>

            <section>
                <h2 class="mb-1.5 px-1 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">{{ t.einstellungen.notizen }}</h2>
                <div class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-muted/40">
                    <SchalterZeile
                        :titel="t.einstellungen.notizBeimStoppen"
                        :text="t.einstellungen.notizBeimStoppenText"
                        :an="einstellungen.notiz_beim_stoppen"
                        @aendern="(an) => aendern({ notiz_beim_stoppen: an })"
                    />
                </div>
            </section>
        </template>

        <section v-else-if="reiter === 'kuerzel'">
            <p class="mb-2.5 px-1 text-[12px] text-muted-foreground">{{ t.einstellungen.kuerzelText }}</p>
            <p v-if="belegt.length > 0" role="alert" class="mb-2 px-1 text-[12px] text-destructive">
                {{ mit(t.einstellungen.kuerzelBelegt, { liste: belegt.map(anzeigen).join(', ') }) }}
            </p>
            <KuerzelListe :kuerzel="einstellungen.kuerzel" :auswahl="auswahl" :belegt="belegt" @aendern="(kuerzel: Kuerzel[]) => aendern({ kuerzel })" />
        </section>

        <template v-else>
            <section>
                <h2 class="mb-1.5 px-1 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">{{ t.update.titel }}</h2>
                <div class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-muted/40">
                    <SchalterZeile :titel="t.update.suchen" :text="t.update.suchenText" :an="einstellungen.updates_suchen" @aendern="(an) => aendern({ updates_suchen: an })" />
                    <div class="flex items-center gap-3 px-3 py-2.5">
                        <span class="min-w-0 flex-1">
                            <span class="block text-[13px] font-medium">{{ t.einstellungen.version }} {{ version }}</span>
                            <span v-if="update" class="block text-[12px] text-muted-foreground">{{ mit(t.update.da, { version: update }) }}</span>
                            <span v-else-if="geprueft" class="block text-[12px] text-muted-foreground">{{ t.update.aktuell }}</span>
                        </span>
                        <button v-if="update" type="button" class="h-8 shrink-0 rounded-md bg-primary px-3 text-[12px] font-medium text-primary-foreground" @click="emit('aktualisieren')">
                            {{ t.update.installieren }}
                        </button>
                        <button
                            v-else
                            type="button"
                            class="h-8 shrink-0 rounded-md border border-border bg-background px-3 text-[12px] font-medium hover:bg-muted disabled:opacity-50"
                            :disabled="sucht"
                            @click="jetztSuchen"
                        >
                            {{ sucht ? t.update.sucht : t.update.jetzt }}
                        </button>
                    </div>
                </div>
            </section>

            <section>
                <h2 class="mb-1.5 px-1 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">{{ t.einstellungen.system }}</h2>
                <div class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-muted/40">
                    <SchalterZeile :titel="t.einstellungen.autostart" :text="t.einstellungen.autostartText" :an="autostart" @aendern="autostartSetzen" />
                </div>
            </section>

            <section>
                <div class="divide-y divide-border overflow-hidden rounded-xl border border-border bg-muted/40">
                    <div class="flex flex-col items-start gap-2 px-3 py-2.5">
                        <span class="text-[12px] leading-snug text-muted-foreground">{{ t.einstellungen.trennenText }}</span>
                        <button type="button" class="h-8 rounded-md border border-border bg-background px-3 text-[12px] font-medium text-destructive hover:bg-muted" @click="verbindungTrennen">
                            {{ t.einstellungen.trennen }}
                        </button>
                    </div>
                </div>
            </section>
        </template>
    </main>
</template>
