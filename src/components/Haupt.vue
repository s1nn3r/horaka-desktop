<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue';
import { alsFehler, aufGezeigt, aufNotizFokus, ausloesen, eintragVerwerfen } from '../befehle';
import { t } from '../i18n';
import { kontextmenue } from '../kontext';
import { useReihenfolge } from '../reihenfolge';
import type { Aktion, Auswahl, Fehler, Kombination, Projekt, Stand, Taetigkeit, Timer } from '../typen';
import { dauer, seit, useUhr } from '../zeit';
import Kopf from './Kopf.vue';
import NotizFeld from './NotizFeld.vue';
import StartFeld from './StartFeld.vue';
import Zeile from './Zeile.vue';

const props = defineProps<{ auswahl: Auswahl; timer: Timer | null; offline: boolean }>();
const emit = defineEmits<{ stand: [stand: Stand]; fehler: [fehler: Fehler]; einstellungen: []; kompakt: [] }>();

const jetzt = useUhr();
const suche = ref('');
const sucheFeld = ref<HTMLInputElement | null>(null);
const offen = ref(new Set<number>());
const beschaeftigt = ref(false);

interface Paar {
    schluessel: string;
    projekt: Projekt;
    taetigkeit: Taetigkeit | null;
}

const projekte = computed(() => new Map(props.auswahl.projects.map((p) => [p.id, p])));

function paar(projektId: number, taetigkeitId: number | null): Paar | null {
    const projekt = projekte.value.get(projektId);
    if (!projekt) {
        return null;
    }
    const taetigkeit = taetigkeitId === null ? null : (projekt.activities.find((a) => a.id === taetigkeitId) ?? null);
    return { schluessel: `${projekt.id}:${taetigkeit?.id ?? 0}`, projekt, taetigkeit };
}

function paare(liste: Kombination[], ohne: Set<string> = new Set()): Paar[] {
    const ergebnis: Paar[] = [];
    for (const k of liste) {
        const p = paar(k.project_id, k.activity_id);
        if (p && !ohne.has(p.schluessel) && !ergebnis.some((e) => e.schluessel === p.schluessel)) {
            ergebnis.push(p);
        }
    }
    return ergebnis;
}

const laufende = computed(() =>
    (props.timer?.entries ?? []).map((e) => ({ eintrag: e, paar: paar(e.project_id, e.activity_id) })),
);
const laufendSchluessel = computed(() => new Set(laufende.value.map((l) => `${l.eintrag.project_id}:${l.eintrag.activity_id ?? 0}`)));
const laufendeProjekte = computed(() => new Set(laufende.value.map((l) => l.eintrag.project_id)));
const irgendwasLaeuft = computed(() => laufende.value.length > 0);

// Favoriten in eigener Reihenfolge (Ziehen), unabhängig von Horaka und der Kompaktansicht.
const reihenfolge = useReihenfolge('haupt');
const favoriten = computed(() => reihenfolge.ordnen(paare(props.auswahl.favorites)));
const zuletzt = computed(() => paare(props.auswahl.recent, new Set(favoriten.value.map((f) => f.schluessel))).slice(0, 5));

function passt(text: string, begriffe: string[]): boolean {
    const klein = text.toLowerCase();
    return begriffe.every((b) => klein.includes(b));
}

// Suche über Kunde, Projekt und Tätigkeit; jedes Wort muss vorkommen.
const treffer = computed<Paar[]>(() => {
    const begriffe = suche.value.toLowerCase().split(/\s+/).filter(Boolean);
    if (begriffe.length === 0) {
        return [];
    }
    const ergebnis: Paar[] = [];
    for (const projekt of props.auswahl.projects) {
        const basis = `${projekt.client_name} ${projekt.name}`;
        if (passt(basis, begriffe)) {
            ergebnis.push({ schluessel: `${projekt.id}:0`, projekt, taetigkeit: null });
        }
        for (const taetigkeit of projekt.activities) {
            if (passt(`${basis} ${taetigkeit.name}`, begriffe) && !passt(basis, begriffe)) {
                ergebnis.push({ schluessel: `${projekt.id}:${taetigkeit.id}`, projekt, taetigkeit });
            }
        }
    }
    return ergebnis;
});

function laeuft(p: Paar): boolean {
    return p.taetigkeit === null ? laufendeProjekte.value.has(p.projekt.id) : laufendSchluessel.value.has(p.schluessel);
}

async function aktion(was: Aktion, projekt: number | null = null, taetigkeit: number | null = null) {
    await ausfuehren(() => ausloesen(was, projekt, taetigkeit));
    if (was !== 'stop') {
        suche.value = '';
    }
}

async function ausfuehren(befehl: () => Promise<Stand>) {
    if (beschaeftigt.value) {
        return;
    }
    beschaeftigt.value = true;
    try {
        emit('stand', await befehl());
    } catch (e) {
        emit('fehler', alsFehler(e));
    } finally {
        beschaeftigt.value = false;
    }
}

function fuerPaar(p: Paar, was: Aktion) {
    void aktion(was, p.projekt.id, p.taetigkeit?.id ?? null);
}

function umschalten(id: number) {
    const neu = new Set(offen.value);
    if (neu.has(id)) {
        neu.delete(id);
    } else {
        neu.add(id);
    }
    offen.value = neu;
}

// Enter startet den ersten Treffer.
function ersterTreffer() {
    const erster = treffer.value[0];
    if (erster) {
        fuerPaar(erster, laeuft(erster) ? 'stop' : 'start');
    }
}

function fokus() {
    void nextTick(() => sucheFeld.value?.focus());
}

// Notizfelder der laufenden Timer; das Kürzel „Notiz“ öffnet das des
// zuletzt gestarteten.
const notizFelder = new Map<number, InstanceType<typeof NotizFeld>>();
function notizFeldMerken(id: number, feld: unknown) {
    if (feld) {
        notizFelder.set(id, feld as InstanceType<typeof NotizFeld>);
    } else {
        notizFelder.delete(id);
    }
}
function notizOeffnen() {
    const letzter = laufende.value[laufende.value.length - 1];
    if (letzter) {
        notizFelder.get(letzter.eintrag.id)?.oeffnen();
    }
}

// Startzeit-Felder der laufenden Timer, geöffnet aus dem Kontextmenü.
const startFelder = new Map<number, InstanceType<typeof StartFeld>>();
function startFeldMerken(id: number, feld: unknown) {
    if (feld) {
        startFelder.set(id, feld as InstanceType<typeof StartFeld>);
    } else {
        startFelder.delete(id);
    }
}

// Rechtsklick auf eine Zeile; „Notiz“ springt in das Feld unter dem laufenden Timer.
function menue(projekt: number, taetigkeit: number | null, lauf = laufende.value.find((l) => l.eintrag.project_id === projekt && (taetigkeit === null || l.eintrag.activity_id === taetigkeit))?.eintrag) {
    kontextmenue(lauf ?? null, {
        starten: () => void aktion('start', projekt, taetigkeit),
        stoppen: () => void aktion('stop', projekt),
        notiz: (e) => notizFelder.get(e.id)?.oeffnen(),
        start: (e) => startFelder.get(e.id)?.oeffnen(),
        loeschen: (e) => void ausfuehren(() => eintragVerwerfen(e.id)),
    });
}

const abmelden: Array<() => void> = [];

onMounted(async () => {
    fokus();
    abmelden.push(await aufGezeigt(fokus));
    abmelden.push(await aufNotizFokus(() => void nextTick(notizOeffnen)));
});

onBeforeUnmount(() => abmelden.forEach((fn) => fn()));
</script>

<template>
    <Kopf @einstellungen="emit('einstellungen')" @kompakt="emit('kompakt')" />

    <!-- Ohne Netz: Hinweis, der von selbst verschwindet, sobald der Abgleich wieder klappt. -->
    <div v-if="offline" role="status" class="border-b border-border bg-muted px-3 py-2 text-[12px] text-muted-foreground">
        {{ t.laden.offline }}
    </div>

    <!-- Laufende Timer mit live tickender Dauer. -->
    <section v-if="irgendwasLaeuft" class="border-b border-border bg-primary-tint px-3 py-2">
        <div class="mb-1 flex items-center justify-between">
            <h2 class="text-[12px] font-semibold tracking-wide text-primary uppercase">{{ t.haupt.laufend }}</h2>
            <button v-if="laufende.length > 1" type="button" class="text-[12px] font-medium text-destructive" :disabled="beschaeftigt" @click="aktion('stop')">
                {{ t.haupt.alleStoppen }}
            </button>
        </div>
        <div v-for="l in laufende" :key="l.eintrag.id" class="py-1" @contextmenu.prevent="menue(l.eintrag.project_id, l.eintrag.activity_id, l.eintrag)">
            <div class="flex min-h-11 items-center gap-2">
                <span class="size-2.5 shrink-0 animate-pulse rounded-full" :style="{ background: l.paar?.projekt.color ?? 'currentColor' }" />
                <div class="min-w-0 flex-1">
                    <div class="truncate font-medium">{{ l.paar?.projekt.name ?? `#${l.eintrag.project_id}` }}</div>
                    <div class="truncate text-[12px] text-muted-foreground">
                        {{ [l.paar?.projekt.client_name, l.paar?.taetigkeit?.name].filter(Boolean).join(' · ') }}
                    </div>
                </div>
                <span class="font-mono text-[15px] font-semibold tabular-nums">{{ dauer(seit(l.eintrag.started_at, jetzt)) }}</span>
                <button
                    type="button"
                    class="flex size-8 shrink-0 items-center justify-center rounded-full bg-destructive text-destructive-foreground"
                    :disabled="beschaeftigt"
                    :title="t.haupt.stoppen"
                    @click="aktion('stop', l.eintrag.project_id)"
                >
                    <svg viewBox="0 0 16 16" class="size-3.5" aria-hidden="true"><rect x="3.5" y="3.5" width="9" height="9" rx="1.5" fill="currentColor" /></svg>
                </button>
            </div>
            <div class="pl-[18px]">
                <StartFeld
                    :ref="(feld) => startFeldMerken(l.eintrag.id, feld)"
                    :id="l.eintrag.id"
                    :start="l.eintrag.started_at"
                    @stand="emit('stand', $event)"
                    @fehler="emit('fehler', $event)"
                />
                <NotizFeld
                    :ref="(feld) => notizFeldMerken(l.eintrag.id, feld)"
                    :id="l.eintrag.id"
                    :notiz="l.eintrag.note"
                    :projekt="l.eintrag.project_id"
                    :taetigkeit="l.eintrag.activity_id"
                    @stand="emit('stand', $event)"
                    @fehler="emit('fehler', $event)"
                />
            </div>
        </div>
    </section>

    <div class="px-3 pt-3 pb-2">
        <input
            ref="sucheFeld"
            v-model="suche"
            type="search"
            :placeholder="t.haupt.suche"
            class="h-10 w-full rounded-lg border border-border bg-muted px-3 outline-none focus:border-primary focus:bg-background"
            @keydown.enter.prevent="ersterTreffer"
        />
    </div>

    <main class="flex-1 overflow-y-auto px-1.5 pb-3">
        <template v-if="suche.trim() !== ''">
            <Zeile
                v-for="p in treffer"
                :key="p.schluessel"
                :projekt="p.projekt"
                :taetigkeit="p.taetigkeit"
                :laeuft="laeuft(p)"
                :parallel="auswahl.parallel_timers"
                :andere-laufen="irgendwasLaeuft"
                :beschaeftigt="beschaeftigt"
                @aktion="fuerPaar(p, $event)"
                @contextmenu.prevent="menue(p.projekt.id, p.taetigkeit?.id ?? null)"
            />
            <p v-if="treffer.length === 0" class="px-2 py-6 text-center text-muted-foreground">{{ t.haupt.keine }}</p>
        </template>

        <template v-else>
            <p v-if="auswahl.projects.length === 0" class="px-2 py-6 text-center text-muted-foreground">{{ t.haupt.leer }}</p>

            <template v-for="gruppe in [{ titel: t.haupt.favoriten, liste: favoriten, ziehbar: true }, { titel: t.haupt.zuletzt, liste: zuletzt, ziehbar: false }]" :key="gruppe.titel">
                <section v-if="gruppe.liste.length > 0" class="mb-2">
                    <h2 class="px-2 pt-2 pb-1 text-[12px] font-semibold tracking-wide text-muted-foreground uppercase">{{ gruppe.titel }}</h2>
                    <Zeile
                        v-for="p in gruppe.liste"
                        :key="p.schluessel"
                        :projekt="p.projekt"
                        :taetigkeit="p.taetigkeit"
                        :laeuft="laeuft(p)"
                        :parallel="auswahl.parallel_timers"
                        :andere-laufen="irgendwasLaeuft"
                        :beschaeftigt="beschaeftigt"
                        :data-schluessel="gruppe.ziehbar ? p.schluessel : undefined"
                        :class="{ 'opacity-50': reihenfolge.ziehend.value === p.schluessel }"
                        @aktion="fuerPaar(p, $event)"
                        @contextmenu.prevent="menue(p.projekt.id, p.taetigkeit?.id ?? null)"
                        @pointerdown="gruppe.ziehbar && reihenfolge.an($event, p.schluessel, favoriten.map((f) => f.schluessel))"
                    />
                </section>
            </template>

            <section v-if="auswahl.projects.length > 0">
                <h2 class="px-2 pt-2 pb-1 text-[12px] font-semibold tracking-wide text-muted-foreground uppercase">{{ t.haupt.projekte }}</h2>
                <template v-for="projekt in auswahl.projects" :key="projekt.id">
                    <Zeile
                        :projekt="projekt"
                        :taetigkeit="null"
                        :laeuft="laufendeProjekte.has(projekt.id)"
                        :parallel="auswahl.parallel_timers"
                        :andere-laufen="irgendwasLaeuft"
                        :beschaeftigt="beschaeftigt"
                        aufklappbar
                        :offen="offen.has(projekt.id)"
                        @aktion="aktion($event, projekt.id)"
                        @contextmenu.prevent="menue(projekt.id, null)"
                        @umschalten="umschalten(projekt.id)"
                    />
                    <div v-if="offen.has(projekt.id)" class="ml-5 border-l border-border pl-1">
                        <Zeile
                            v-for="taetigkeit in projekt.activities"
                            :key="taetigkeit.id"
                            :projekt="projekt"
                            :taetigkeit="taetigkeit"
                            :laeuft="laufendSchluessel.has(`${projekt.id}:${taetigkeit.id}`)"
                            :parallel="auswahl.parallel_timers"
                            :andere-laufen="irgendwasLaeuft"
                            :beschaeftigt="beschaeftigt"
                            @aktion="aktion($event, projekt.id, taetigkeit.id)"
                            @contextmenu.prevent="menue(projekt.id, taetigkeit.id)"
                        />
                    </div>
                </template>
            </section>
        </template>
    </main>
</template>
