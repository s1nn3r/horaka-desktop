<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue';
import { kuerzelFortsetzen, kuerzelPausieren } from '../befehle';
import { t } from '../i18n';
import { anzeigen, ausTaste, gleich } from '../kuerzel';
import type { Auswahl, Kuerzel, KuerzelAktion } from '../typen';

// Globale Tastenkürzel: Liste mit Entfernen, darunter ein neues anlegen
// (Aktion, bei Projekt-Aktionen das Ziel, Tasten aufnehmen). Während der
// Aufnahme sind die alten Kürzel abgemeldet, damit sie nicht auslösen.
const props = defineProps<{ kuerzel: Kuerzel[]; auswahl: Auswahl | null; belegt: string[] }>();
const emit = defineEmits<{ aendern: [kuerzel: Kuerzel[]] }>();

const AKTIONEN: KuerzelAktion[] = ['fenster', 'notiz', 'stoppen', 'umschalten', 'wechseln'];

const neu = ref(false);
const aktion = ref<KuerzelAktion>('umschalten');
const ziel = ref('');
const taste = ref<string | null>(null);
const nimmtAuf = ref(false);
const hinweis = ref<string | null>(null);

const braucheZiel = computed(() => aktion.value === 'umschalten' || aktion.value === 'wechseln');

// Ziele: jedes Projekt allein und mit jeder seiner Tätigkeiten.
const ziele = computed(() =>
    (props.auswahl?.projects ?? []).flatMap((p) => [
        { wert: `${p.id}:0`, text: `${p.client_name} · ${p.name}` },
        ...p.activities.map((a) => ({ wert: `${p.id}:${a.id}`, text: `${p.client_name} · ${p.name} · ${a.name}` })),
    ]),
);

function zielText(k: Kuerzel): string {
    const p = props.auswahl?.projects.find((x) => x.id === k.projekt);
    if (!p) return k.projekt ? `#${k.projekt}` : '';
    const a = p.activities.find((x) => x.id === k.taetigkeit);
    return [p.client_name, p.name, a?.name].filter(Boolean).join(' · ');
}

async function aufnehmen() {
    hinweis.value = null;
    taste.value = null;
    nimmtAuf.value = true;
    await kuerzelPausieren();
}

async function aufnahmeEnde() {
    if (nimmtAuf.value) {
        nimmtAuf.value = false;
        await kuerzelFortsetzen();
    }
}

function tasteGedrueckt(e: KeyboardEvent) {
    if (!nimmtAuf.value) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape' && !e.ctrlKey && !e.altKey && !e.metaKey) {
        void aufnahmeEnde();
        return;
    }
    const kuerzel = ausTaste(e);
    if (kuerzel === null) {
        // Nur Modifikator gedrückt oder Taste ohne Modifikator: weiter warten.
        if (!['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) hinweis.value = t.einstellungen.kuerzelRegel;
        return;
    }
    if (props.kuerzel.some((k) => gleich(k.kuerzel, kuerzel))) {
        hinweis.value = t.einstellungen.kuerzelDoppelt;
        return;
    }
    hinweis.value = null;
    taste.value = kuerzel;
    void aufnahmeEnde();
}

function hinzufuegen() {
    if (!taste.value || (braucheZiel.value && !ziel.value)) return;
    const [p, a] = ziel.value.split(':').map(Number);
    emit('aendern', [
        ...props.kuerzel,
        {
            kuerzel: taste.value,
            aktion: aktion.value,
            projekt: braucheZiel.value ? p : null,
            taetigkeit: braucheZiel.value && a > 0 ? a : null,
        },
    ]);
    neu.value = false;
    taste.value = null;
    ziel.value = '';
}

function entfernen(index: number) {
    emit(
        'aendern',
        props.kuerzel.filter((_, i) => i !== index),
    );
}

onBeforeUnmount(() => void aufnahmeEnde());
</script>

<template>
    <div class="flex flex-col gap-2">
        <ul v-if="kuerzel.length > 0" class="divide-y divide-border rounded-lg border border-border">
            <li v-for="(k, i) in kuerzel" :key="k.kuerzel" class="flex items-center gap-2 px-2.5 py-2">
                <kbd
                    class="shrink-0 rounded border px-1.5 py-0.5 font-mono text-[12px]"
                    :class="belegt.some((b) => gleich(b, k.kuerzel)) ? 'border-destructive text-destructive' : 'border-border bg-muted'"
                >
                    {{ anzeigen(k.kuerzel) }}
                </kbd>
                <div class="min-w-0 flex-1 text-[12px]">
                    <div class="truncate font-medium">{{ t.einstellungen.aktionen[k.aktion] }}</div>
                    <div v-if="k.projekt" class="truncate text-muted-foreground">{{ zielText(k) }}</div>
                </div>
                <button
                    type="button"
                    class="flex size-7 shrink-0 items-center justify-center rounded-md text-muted-foreground hover:bg-muted hover:text-destructive"
                    :title="t.einstellungen.kuerzelEntfernen"
                    @click="entfernen(i)"
                >
                    <svg viewBox="0 0 16 16" class="size-3.5" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" /></svg>
                </button>
            </li>
        </ul>
        <p v-else class="text-[12px] text-muted-foreground">{{ t.einstellungen.kuerzelLeer }}</p>

        <div v-if="neu" class="flex flex-col gap-2 rounded-lg border border-border p-2.5">
            <label class="flex flex-col gap-1 text-[12px] font-medium">
                {{ t.einstellungen.kuerzelAktion }}
                <select v-model="aktion" class="h-9 rounded-md border border-border bg-background px-2 font-normal">
                    <option v-for="a in AKTIONEN" :key="a" :value="a">{{ t.einstellungen.aktionen[a] }}</option>
                </select>
            </label>
            <label v-if="braucheZiel" class="flex flex-col gap-1 text-[12px] font-medium">
                {{ t.einstellungen.kuerzelZiel }}
                <select v-model="ziel" class="h-9 rounded-md border border-border bg-background px-2 font-normal">
                    <option value="" disabled>{{ t.einstellungen.projektWaehlen }}</option>
                    <option v-for="z in ziele" :key="z.wert" :value="z.wert">{{ z.text }}</option>
                </select>
            </label>
            <div class="flex flex-col gap-1 text-[12px] font-medium">
                {{ t.einstellungen.kuerzelTaste }}
                <button
                    type="button"
                    class="h-9 rounded-md border px-2 text-left font-mono font-normal"
                    :class="nimmtAuf ? 'border-primary bg-primary-tint' : 'border-border'"
                    @click="aufnehmen"
                    @keydown="tasteGedrueckt"
                    @blur="aufnahmeEnde"
                >
                    {{ nimmtAuf ? t.einstellungen.kuerzelDruecken : taste ? anzeigen(taste) : t.einstellungen.kuerzelAufnehmen }}
                </button>
                <span v-if="hinweis" class="font-normal text-destructive">{{ hinweis }}</span>
                <span v-else class="font-normal text-muted-foreground">{{ t.einstellungen.kuerzelRegel }}</span>
            </div>
            <div class="flex justify-end gap-2">
                <button type="button" class="h-8 rounded-md px-2.5 text-[12px] font-medium hover:bg-muted" @click="neu = false">{{ t.haupt.abbrechen }}</button>
                <button
                    type="button"
                    class="h-8 rounded-md bg-primary px-3 text-[12px] font-medium text-primary-foreground disabled:opacity-50"
                    :disabled="!taste || (braucheZiel && !ziel)"
                    @click="hinzufuegen"
                >
                    {{ t.einstellungen.kuerzelHinzufuegen }}
                </button>
            </div>
        </div>
        <button v-else type="button" class="h-9 self-start rounded-lg border border-border px-3 text-[13px] font-medium hover:bg-muted" @click="neu = true">
            + {{ t.einstellungen.kuerzelNeu }}
        </button>
    </div>
</template>
