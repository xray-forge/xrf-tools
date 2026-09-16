import { createApplicationTheme } from "./creation";
import { getWashImage } from "./surface";
import { LAYOUT } from "./tokens";

/** Serializes one of MUI's generated style objects, which arrive as selector-keyed rule trees rather than as text. */
function toCssRules(rule: Record<string, unknown>, selector?: string): string {
  const declarations: Array<string> = [];
  const nested: Array<string> = [];

  for (const [key, value] of Object.entries(rule)) {
    if (value !== null && typeof value === "object") {
      nested.push(toCssRules(value as Record<string, unknown>, key));
    } else if (value !== undefined) {
      declarations.push(`${key}: ${String(value)};`);
    }
  }

  const body: string = declarations.length > 0 ? `${selector ?? ":root"} { ${declarations.join(" ")} }` : "";

  return [body, ...nested].filter(Boolean).join("\n");
}

/**
 * Resolves the theme at build time so the first paint does not wait for React or MUI.
 *
 * Emits the application's own stylesheet rather than a parallel vocabulary of its own, so a level that exists in React
 * cannot fail to exist here.
 */
export function getPreloadThemeCss(): string {
  const sheets: Array<Record<string, unknown>> = createApplicationTheme().generateStyleSheets();

  return [
    ...sheets.map((sheet: Record<string, unknown>) => toCssRules(sheet)),
    `html {
      --preload-title-bar-height: ${LAYOUT.titleBarHeight}px;
      --preload-status-bar-height: ${LAYOUT.statusBarHeight}px;
      --preload-rail-width: ${LAYOUT.railWidth}px;
      --preload-surface-radius: ${LAYOUT.surfaceRadius}px;
      --xrf-wash: ${getWashImage("dark")};
      color-scheme: dark;
    }`,
    `html[data-color-scheme="light"] {
      --xrf-wash: ${getWashImage("light")};
      color-scheme: light;
    }`,
  ].join("\n");
}
