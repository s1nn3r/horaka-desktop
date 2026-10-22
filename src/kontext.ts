import { Menu } from '@tauri-apps/api/menu';
import { t } from './i18n';
import type { Eintrag } from './typen';

// Rechtsklick auf eine Kachel oder Zeile: natives Menü (wird nicht vom
// kleinen Fenster abgeschnitten). Läuft sie nicht: Starten; läuft sie:
// Stoppen, Notiz, Startzeit, Löschen (verwirft den Eintrag ohne Rückfrage).
export function kontextmenue(
    lauf: Eintrag | null,
    a: { starten: () => void; stoppen: () => void; notiz: (e: Eintrag) => void; start: (e: Eintrag) => void; loeschen: (e: Eintrag) => void },
) {
    const punkte = lauf
        ? [
              { text: t.haupt.stoppen, action: a.stoppen },
              { text: t.haupt.notizHinzufuegen, action: () => a.notiz(lauf) },
              { text: t.haupt.startKorrigieren, action: () => a.start(lauf) },
              { item: 'Separator' as const },
              { text: t.haupt.loeschen, action: () => a.loeschen(lauf) },
          ]
        : [{ text: t.haupt.starten, action: a.starten }];
    void Menu.new({ items: punkte }).then((m) => m.popup());
}
