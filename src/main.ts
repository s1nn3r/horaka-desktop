import { invoke } from '@tauri-apps/api/core';
import { createApp } from 'vue';
import { spracheSetzen } from './i18n';
import './style.css';
import type { Einstellungen } from './typen';

async function starten() {
    // Vorschau im Browser mit Beispieldaten, nur beim Entwickeln.
    if (import.meta.env.DEV && new URLSearchParams(location.search).has('vorschau')) {
        await import('./vorschau');
    }
    // Druck auf eine Kopfzeile (data-tauri-drag-region): Rust erfährt, dass
    // gezogen wird, und schließt das Fenster dabei nicht wie beim Wegklicken.
    // In der Erfassungsphase, also vor dem Ziehen, das Tauri selbst startet.
    document.addEventListener(
        'mousedown',
        (e) => {
            if (e.button === 0 && e.target instanceof Element && e.target.hasAttribute('data-tauri-drag-region')) {
                void invoke('ziehen_beginnt').catch(() => undefined);
            }
        },
        true,
    );

    // Sprache aus den Einstellungen, bevor die Oberfläche Texte liest.
    const einstellungen = await invoke<Einstellungen>('einstellungen_lesen').catch(() => null);
    spracheSetzen(einstellungen?.sprache ?? 'system');
    const { default: App } = await import('./App.vue');

    createApp(App).mount('#app');
}

void starten();
