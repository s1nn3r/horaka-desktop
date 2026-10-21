// Projektfarbe aus Horaka: die API liefert einen Schlüssel der Palette
// (teal, amber, …), die Hexwerte je Thema stehen als --projekt-* in style.css.
// Ein Hexwert (Vorschau, ältere Daten) gilt unverändert.
export function projektfarbe(wert: string | null | undefined, ersatz = 'currentColor'): string {
    if (!wert) {
        return ersatz;
    }
    return wert.startsWith('#') ? wert : `var(--projekt-${wert}, ${ersatz})`;
}
