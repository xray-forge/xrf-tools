import { ColorScheme } from "#/theme/tokens";

/**
 * The attribute on the document element naming the colour scheme shown: MUI writes it and the theme's rules read it.
 * `main-init.ts` spells it out too, being inlined into the page before any module loads.
 */
export const COLOR_SCHEME_ATTRIBUTE: string = "data-color-scheme";

/**
 * @param scheme - A colour scheme.
 * @returns The selector matching the document while it shows that scheme.
 */
export function toColorSchemeSelector(scheme: ColorScheme): string {
  return `[${COLOR_SCHEME_ATTRIBUTE}="${scheme}"]`;
}

/**
 * @returns The colour scheme the document shows, dark until one is set, as the theme defaults to.
 */
export function readColorScheme(): ColorScheme {
  return document.documentElement.getAttribute(COLOR_SCHEME_ATTRIBUTE) === "light" ? "light" : "dark";
}

/**
 * Tells a listener the colour scheme each time the document's changes, before the page paints in it, until stopped.
 *
 * @param listener - Hears the scheme now shown.
 * @returns What stops it.
 */
export function watchColorScheme(listener: (scheme: ColorScheme) => void): () => void {
  const observer: MutationObserver = new MutationObserver(() => listener(readColorScheme()));

  observer.observe(document.documentElement, { attributeFilter: [COLOR_SCHEME_ATTRIBUTE], attributes: true });

  return () => observer.disconnect();
}
