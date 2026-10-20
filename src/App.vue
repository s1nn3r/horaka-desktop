<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import {
    abschlussUeberspringen,
    alsFehler,
    aufFehler,
    aufEinstellungenOeffnen,
    aufGezeigt,
    aufMaus,
    aufStand,
    einstellungenLesen,
    einstellungenSpeichern,
    groesseSetzen,
    neuLaden,
    standLesen,
    updateInstallieren,
    verbergen,
} from './befehle';
import Abschluss from './components/Abschluss.vue';
import EinstellungenAnsicht from './components/Einstellungen.vue';
import Haupt from './components/Haupt.vue';
import Kompakt from './components/Kompakt.vue';
import Laden from './components/Laden.vue';
import Verbinden from './components/Verbinden.vue';
import { fehlertext, mit, t } from './i18n';
import { themaAnwenden } from './thema';
import type { Einstellungen, Fehler, Stand } from './typen';

const stand = ref<Stand | null>(null);
const einstellungen = ref<Einstellungen | null>(null);
const belegt = ref<string[]>([]);
// Nach dem Neuladen beim Sprachwechsel wieder in den Einstellungen.
const ansicht = ref<'haupt' | 'einstellungen'>(sessionStorage.getItem('ansicht') === 'einstellungen' ? 'einstellungen' : 'haupt');
sessionStorage.removeItem('ansicht');
const rahmen = ref<HTMLElement | null>(null);
const meldung = ref<string | null>(null);
let meldungTakt = 0;
const abmelden: Array<() => void> = [];

// Fehlendes Netz ist ein Zustand (Stand.offline, Hinweis oben, geht von
// selbst weg), keine Meldung, die stehen bleibt.
function zeigeFehler(f: Fehler) {
    if (f.code === 'network') {
        return;
    }
    meldung.value = fehlertext(f.code, f.detail);
    window.clearTimeout(meldungTakt);
    meldungTakt = window.setTimeout(() => (meldung.value = null), 8000);
}

function uebernehmen(neu: Stand) {
    stand.value = neu;
    if (!neu.verbunden) {
        ansicht.value = 'haupt';
    }
}

async function laden() {
    try {
        uebernehmen(await neuLaden());
    } catch (e) {
        zeigeFehler(alsFehler(e));
        stand.value = await standLesen();
    }
}

async function einstellungenAendern(neu: Einstellungen) {
    const spracheNeu = neu.sprache !== einstellungen.value?.sprache;
    einstellungen.value = neu;
    themaAnwenden(neu.thema);
    const antwort = await einstellungenSpeichern(neu);
    einstellungen.value = antwort.einstellungen;
    belegt.value = antwort.belegt;
    // Die Texte stehen beim Start fest: neu laden, in den Einstellungen bleiben.
    if (spracheNeu) {
        sessionStorage.setItem('ansicht', ansicht.value);
        location.reload();
    }
}

function kompaktSetzen(an: boolean) {
    if (einstellungen.value) {
        void einstellungenAendern({ ...einstellungen.value, kompakt: an });
    }
}

// Kompakt nur in der Hauptansicht mit Daten; Einstellungen, Verbinden und die
// Frage nach der Notiz brauchen das große Fenster.
const kompaktAktiv = computed(
    () =>
        !!einstellungen.value?.kompakt &&
        !!stand.value?.verbunden &&
        !!stand.value.auswahl &&
        stand.value.abschluss.length === 0 &&
        ansicht.value === 'haupt',
);

// Ausblenden: Rust meldet die Maus auch ohne Fokus (`maus`), die Ereignisse
// im Fenster selbst reagieren sofort.
const drueber = ref(true);
const ausgeblendet = computed(() => kompaktAktiv.value && !!einstellungen.value?.kompakt_durchsichtig && !drueber.value);
// Während der Regler in den Einstellungen gezogen wird, zeigt das ganze
// Fenster die gewählte Deckkraft (ohne Übergang, damit es dem Regler folgt).
const vorschauDeckkraft = ref<number | null>(null);
const deckkraft = computed(() => {
    if (vorschauDeckkraft.value !== null) {
        return vorschauDeckkraft.value / 100;
    }
    return ausgeblendet.value ? (einstellungen.value?.kompakt_deckkraft ?? 30) / 100 : 1;
});
const mausRein = () => (drueber.value = true);
const mausRaus = () => (drueber.value = false);

const BREITE = 360;
const HOEHE = 560;
// Kompakt: Breite in ganzen Kachelspalten (Kachel 64 px, Abstand 4 px, Rand
// und Symbolspalte 36 px), am rechten Rand ziehbar; die Zeilen folgen daraus.
// shortcut: Spaltenzahl nur in localStorage dieses Fensters, in einstellungen.json, falls sie mit umziehen soll.
const SPALTEN_MIN = 2;
const SPALTEN_MAX = 10;
function spaltenGemerkt(): number {
    try {
        const n = Number(localStorage.getItem('kompakt-spalten'));
        return n >= SPALTEN_MIN && n <= SPALTEN_MAX ? n : 3;
    } catch {
        return 3;
    }
}
const spalten = ref(spaltenGemerkt());
const kompaktBreite = computed(() => 36 + spalten.value * 68 - 4);
function spaltenSetzen(n: number) {
    spalten.value = Math.min(SPALTEN_MAX, Math.max(SPALTEN_MIN, n));
    try {
        localStorage.setItem('kompakt-spalten', String(spalten.value));
    } catch {
        // Ohne Speicher gilt die Breite nur bis zum Neustart.
    }
}
watch(spalten, () => {
    if (kompaktAktiv.value && rahmen.value) {
        void groesseSetzen(kompaktBreite.value, Math.ceil(rahmen.value.getBoundingClientRect().height));
    }
});
let beobachter: ResizeObserver | null = null;

watch(kompaktAktiv, async (an) => {
    beobachter?.disconnect();
    beobachter = null;
    if (!an) {
        await groesseSetzen(BREITE, HOEHE);
        return;
    }
    await nextTick();
    if (rahmen.value) {
        // Höhe folgt dem Inhalt (laufende Timer, Meldungen).
        beobachter = new ResizeObserver(([e]) => void groesseSetzen(kompaktBreite.value, Math.ceil(e.target.getBoundingClientRect().height)));
        beobachter.observe(rahmen.value);
    }
});

// „Nicht mehr fragen“ im Abschluss: Einstellung aus, offene Fragen weg.
async function nichtMehrFragen() {
    if (einstellungen.value) {
        await einstellungenAendern({ ...einstellungen.value, notiz_beim_stoppen: false });
    }
    for (const g of stand.value?.abschluss ?? []) {
        uebernehmen(await abschlussUeberspringen(g.id));
    }
}

async function aktualisieren() {
    try {
        await updateInstallieren();
    } catch (e) {
        zeigeFehler(alsFehler(e));
    }
}

function taste(e: KeyboardEvent) {
    if (e.key === 'Escape') {
        if (ansicht.value === 'einstellungen') {
            ansicht.value = 'haupt';
        } else {
            void verbergen();
        }
    }
}

onMounted(async () => {
    window.addEventListener('keydown', taste);
    document.documentElement.addEventListener('mouseenter', mausRein);
    document.documentElement.addEventListener('mouseleave', mausRaus);
    abmelden.push(await aufMaus((ja) => (drueber.value = ja)));
    // Rechtsklick auf das Symbol, „Einstellungen“ (nur mit Verbindung sinnvoll).
    abmelden.push(
        await aufEinstellungenOeffnen(() => {
            if (stand.value?.verbunden) {
                ansicht.value = 'einstellungen';
            }
        }),
    );
    einstellungen.value = await einstellungenLesen();
    themaAnwenden(einstellungen.value.thema);
    abmelden.push(await aufStand(uebernehmen));
    abmelden.push(await aufFehler(zeigeFehler));
    abmelden.push(await aufGezeigt(() => void laden()));
    stand.value = await standLesen();
    await laden();
});

onBeforeUnmount(() => {
    beobachter?.disconnect();
    window.removeEventListener('keydown', taste);
    document.documentElement.removeEventListener('mouseenter', mausRein);
    document.documentElement.removeEventListener('mouseleave', mausRaus);
    abmelden.forEach((fn) => fn());
});
</script>

<template>
    <div ref="rahmen" class="flex flex-col overflow-hidden motion-reduce:transition-none"
        :class="[kompaktAktiv ? 'rounded-lg bg-muted' : 'h-full border border-border bg-background', vorschauDeckkraft === null ? 'transition-opacity duration-300 ease-out' : '']"
        :style="{ opacity: deckkraft }"
    >
        <template v-if="stand && einstellungen">
            <Verbinden v-if="!stand.verbunden" @verbunden="uebernehmen" @fehler="zeigeFehler" />
            <EinstellungenAnsicht
                @deckkraft-vorschau="vorschauDeckkraft = $event"
                v-else-if="ansicht === 'einstellungen'"
                :einstellungen="einstellungen"
                :belegt="belegt"
                :auswahl="stand.auswahl"
                :update="stand.update"
                @stand="uebernehmen"
                @aktualisieren="aktualisieren"
                @zurueck="ansicht = 'haupt'"
                @getrennt="uebernehmen"
                @fehler="zeigeFehler"
                @aendern="einstellungenAendern"
            />
            <Abschluss
                v-else-if="stand.abschluss.length > 0"
                :abschluss="stand.abschluss"
                :auswahl="stand.auswahl"
                @stand="uebernehmen"
                @fehler="zeigeFehler"
                @einstellungen="ansicht = 'einstellungen'"
                @nicht-mehr-fragen="nichtMehrFragen"
            />
            <Kompakt
                v-else-if="kompaktAktiv && stand.auswahl"
                :auswahl="stand.auswahl"
                :timer="stand.timer"
                :offline="stand.offline"
                :spalten="spalten"
                @stand="uebernehmen"
                @fehler="zeigeFehler"
                @gross="kompaktSetzen(false)"
                @spalten="spaltenSetzen"
            />
            <Haupt
                v-else-if="stand.auswahl"
                :auswahl="stand.auswahl"
                :timer="stand.timer"
                :offline="stand.offline"
                @stand="uebernehmen"
                @fehler="zeigeFehler"
                @einstellungen="ansicht = 'einstellungen'"
                @kompakt="kompaktSetzen(true)"
            />
            <Laden v-else :offline="stand.offline" @erneut="laden" @einstellungen="ansicht = 'einstellungen'" />
        </template>

        <div
            v-if="stand?.update"
            role="status"
            class="flex items-center gap-2 border border-primary/40 bg-primary-tint"
            :class="kompaktAktiv ? 'mx-1 mb-1 rounded-md px-2 py-1 text-[11px]' : 'mx-3 mb-3 rounded-lg px-3 py-2 text-[13px]'"
        >
            <span class="flex-1">{{ stand.update_laeuft ? t.update.laeuft : mit(t.update.da, { version: stand.update }) }}</span>
            <button
                v-if="!stand.update_laeuft"
                type="button"
                class="shrink-0 rounded-md bg-primary font-medium text-primary-foreground"
                :class="kompaktAktiv ? 'h-6 px-2 text-[11px]' : 'h-8 px-2.5 text-[12px]'"
                @click="aktualisieren"
            >
                {{ t.update.installieren }}
            </button>
        </div>

        <div
            v-if="meldung"
            role="alert"
            class="flex items-start gap-2 bg-destructive text-destructive-foreground"
            :class="kompaktAktiv ? 'mx-1 mb-1 rounded-md px-2 py-1 text-[11px]' : 'mx-3 mb-3 rounded-lg px-3 py-2 text-[13px]'"
        >
            <span class="flex-1">{{ meldung }}</span>
            <button type="button" class="-mr-1 shrink-0 px-1 font-semibold" :title="t.haupt.schliessen" @click="meldung = null">×</button>
        </div>
    </div>
</template>
