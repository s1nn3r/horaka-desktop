import type { Thema } from './typen';

/** Hell, dunkel oder wie das System (style.css liest data-theme). */
export function themaAnwenden(thema: Thema) {
    const wurzel = document.documentElement;
    if (thema === 'system') {
        delete wurzel.dataset.theme;
    } else {
        wurzel.dataset.theme = thema === 'dunkel' ? 'dark' : 'light';
    }
}
