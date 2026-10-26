<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import {
    abschlussUeberspringen,
    alsFehler,
    aufFehler,
    aufEinstellungenOeffnen,
    aufGezeigt,
    aufGrossOeffnen,
    aufNotizFokus,
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
import NotizWahl from './components/NotizWahl.vue';
import Verbinden from './components/Verbinden.vue';
import { fehlertext, mit, t } from './i18n';
import { themaAnwenden } from './thema';
import type { Eintrag, Einstellungen, Fehler, Gestoppt, Stand } from './typen';

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

// Notiz eines laufenden Timers aus dem Kontextmenü der Kompaktansicht: im
// Notizfenster (Abschluss) des großen Fensters, danach wieder kompakt.
const notizEintrag = ref<Gestoppt | null>(null);
function notizOeffnen(e: Eintrag) {
    const sekunden = Math.floor((Date.now() - Date.parse(e.started_at)) / 1000);
    notizEintrag.value = { id: e.id, project_id: e.project_id, activity_id: e.activity_id, note: e.note, sekunden, started_at: e.started_at, ended_at: new Date().toISOString() };
}
// Kürzel „Notiz“: ein laufender Timer gleich ins Notizfenster, mehrere zur Auswahl.
const notizWahl = ref(false);
function notizKuerzel() {
    const laufende = stand.value?.verbunden ? (stand.value.timer?.entries ?? []) : [];
    ansicht.value = 'haupt';
    notizWahl.value = laufende.length > 1;
    if (laufende.length === 1) {
        notizOeffnen(laufende[0]);
    }
}
function notizGewaehlt(e: Eintrag) {
    notizWahl.value = false;
    notizOeffnen(e);
}
function notizFertig(neu: Stand) {
    notizEintrag.value = null;
    uebernehmen(neu);
}

// Kompakt nur in der Hauptansicht mit Daten; Einstellungen, Verbinden und die
// Frage nach der Notiz brauchen das große Fenster.
const kompaktAktiv = computed(
    () =>
        !notizEintrag.value &&
        !notizWahl.value &&
        !!einstellungen.value?.kompakt &&
        !!stand.value?.verbunden &&
        !!stand.value.auswahl &&
        stand.value.abschluss.length === 0 &&
        ansicht.value === 'haupt',
);

// Ausblenden: Rust meldet die Maus auch ohne Fokus (`maus`), die Ereignisse
// im Fenster selbst reagieren sofort.
const drueber = ref(true);
// Beim Ziehen der Breite ist die Maus oft außerhalb: dann nicht ausblenden.
const zieht = ref(false);
const ausgeblendet = computed(() => kompaktAktiv.value && !!einstellungen.value?.kompakt_durchsichtig && !drueber.value && !zieht.value);
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
// Höhe der vollen Ansicht, unten am Anfasser ziehbar (in localStorage wie die Spalten).
const HOEHE_MIN = 400;
const HOEHE_MAX = 1000;
function hoeheGemerkt(): number {
    try {
        const n = Number(localStorage.getItem('fenster-hoehe'));
        return n >= HOEHE_MIN && n <= HOEHE_MAX ? n : 560;
    } catch {
        return 560;
    }
}
const hoehe = ref(hoeheGemerkt());
function hoeheSetzen(n: number) {
    hoehe.value = Math.round(Math.min(HOEHE_MAX, Math.max(HOEHE_MIN, n)));
    void groesseSetzen(BREITE, hoehe.value);
    try {
        localStorage.setItem('fenster-hoehe', String(hoehe.value));
    } catch {
        // Ohne Speicher gilt die Höhe nur bis zum Neustart.
    }
}
let hoeheAb: { y: number; hoehe: number } | null = null;
function hoeheAn(e: PointerEvent) {
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    hoeheAb = { y: e.screenY, hoehe: hoehe.value };
    zieht.value = true;
}
function hoeheZiehen(e: PointerEvent) {
    if (hoeheAb) {
        hoeheSetzen(hoeheAb.hoehe + e.screenY - hoeheAb.y);
    }
}
function hoeheAus() {
    hoeheAb = null;
    zieht.value = false;
}
// Kompakt: Breite in ganzen Kachelspalten (Kachel 112 px, Abstand 8 px; Innenrand
// und Symbolspalte 56 px), am rechten Rand ziehbar; die Zeilen folgen daraus.
// shortcut: Spaltenzahl nur in localStorage dieses Fensters, in einstellungen.json, falls sie mit umziehen soll.
const SPALTEN_MIN = 1;
const SPALTEN_MAX = 5;
function spaltenGemerkt(): number {
    try {
        const n = Number(localStorage.getItem('kompakt-spalten'));
        return n >= SPALTEN_MIN && n <= SPALTEN_MAX ? n : 2;
    } catch {
        return 2;
    }
}
const spalten = ref(spaltenGemerkt());
const kompaktBreite = computed(() => 56 + spalten.value * 120 - 8);
function spaltenSetzen(n: number) {
    spalten.value = Math.min(SPALTEN_MAX, Math.max(SPALTEN_MIN, n));
    try {
        localStorage.setItem('kompakt-spalten', String(spalten.value));
    } catch {
        // Ohne Speicher gilt die Breite nur bis zum Neustart.
    }
}
// Kompaktansicht und Notizfenster (Abschluss) sind so hoch wie ihr Inhalt,
// sonst gilt die gemerkte Höhe.
const abschlussAktiv = computed(
    () => !!stand.value?.verbunden && ansicht.value === 'haupt' && (!!notizEintrag.value || notizWahl.value || stand.value.abschluss.length > 0),
);
const inhaltHoch = computed(() => kompaktAktiv.value || abschlussAktiv.value);
const fensterBreite = computed(() => (kompaktAktiv.value ? kompaktBreite.value : BREITE));
let beobachter: ResizeObserver | null = null;

watch([inhaltHoch, fensterBreite], async ([an]) => {
    beobachter?.disconnect();
    beobachter = null;
    if (!an) {
        await groesseSetzen(BREITE, hoehe.value);
        return;
    }
    await nextTick();
    if (rahmen.value) {
        // Höhe folgt dem Inhalt (laufende Timer, Meldungen); meldet sich auch gleich beim Beobachten.
        beobachter = new ResizeObserver(([e]) => void groesseSetzen(fensterBreite.value, Math.ceil(e.target.getBoundingClientRect().height)));
        beobachter.observe(rahmen.value);
    }
});

// „Nicht mehr fragen“ im Abschluss: Einstellung aus, offene Fragen weg.
async function nichtMehrFragen() {
    if (einstellungen.value) {
        await einstellungenAendern({ ...einstellungen.value, notiz_beim_stoppen: false });
    }
    await abschlussSchliessen();
}

// Esc im Notizfenster: ohne Notiz weiter für alle offenen Fragen, zurück zur
// vorigen Ansicht (kompakt oder voll; hatte erst die Frage das Fenster
// geöffnet, geht es wieder zu, siehe abschluss_erledigt in Rust).
async function abschlussSchliessen() {
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
        if (notizEintrag.value || notizWahl.value) {
            notizEintrag.value = null;
            notizWahl.value = false;
        } else if (ansicht.value === 'einstellungen') {
            ansicht.value = 'haupt';
        } else if (abschlussAktiv.value) {
            void abschlussSchliessen();
        } else {
            void verbergen();
        }
    }
}

onMounted(async () => {
    // Gemerkte Höhe; die Kompaktansicht setzt ihre Größe danach selbst.
    void groesseSetzen(BREITE, hoehe.value);
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
    abmelden.push(await aufNotizFokus(notizKuerzel));
    einstellungen.value = await einstellungenLesen();
    themaAnwenden(einstellungen.value.thema);
    abmelden.push(await aufStand(uebernehmen));
    abmelden.push(await aufFehler(zeigeFehler));
    abmelden.push(await aufGezeigt(() => void laden()));
    // „Fenster öffnen“ im Tray-Menü: aus der Kompaktansicht in die volle.
    abmelden.push(
        await aufGrossOeffnen(() => {
            if (einstellungen.value?.kompakt) kompaktSetzen(false);
        }),
    );
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
    <div ref="rahmen" class="relative flex flex-col overflow-hidden motion-reduce:transition-none"
        :class="[kompaktAktiv ? 'rounded-lg bg-flyout shadow-[inset_0_0_0_var(--haarlinie)_var(--linie)]' : 'border border-border bg-background', inhaltHoch ? '' : 'h-full', vorschauDeckkraft === null ? 'transition-opacity duration-300 ease-out' : '']"
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
            <NotizWahl v-else-if="notizWahl" :eintraege="stand.timer?.entries ?? []" :auswahl="stand.auswahl" @waehlen="notizGewaehlt" />
            <Abschluss
                v-else-if="notizEintrag"
                :key="`notiz-${notizEintrag.id}`"
                :abschluss="[notizEintrag]"
                :auswahl="stand.auswahl"
                laufend
                @stand="notizFertig"
                @fehler="zeigeFehler"
                @abbrechen="notizEintrag = null"
                @einstellungen="ansicht = 'einstellungen'"
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
                @ziehen="zieht = $event"
                @notiz="notizOeffnen"
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

        <div
            v-if="!inhaltHoch"
            role="separator"
            aria-orientation="horizontal"
            tabindex="0"
            :aria-label="t.haupt.hoehe"
            :aria-valuenow="hoehe"
            class="absolute inset-x-0 bottom-0 h-1.5 cursor-ns-resize focus-visible:bg-ring/40"
            @pointerdown="hoeheAn"
            @pointermove="hoeheZiehen"
            @pointerup="hoeheAus"
            @pointercancel="hoeheAus"
            @lostpointercapture="hoeheAus"
            @keydown.up.prevent="hoeheSetzen(hoehe - 40)"
            @keydown.down.prevent="hoeheSetzen(hoehe + 40)"
        />
    </div>
</template>
