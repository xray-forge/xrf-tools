import { Theme } from "@mui/material";

import { createApplicationTheme } from "./creation";
import { getWashImage } from "./surface";
import { getThemeVariables } from "./variables";

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

function toDeclarations(variables: Record<string, string>): string {
  return Object.entries(variables)
    .map(([name, value]) => `${name}: ${value};`)
    .join("\n      ");
}

/**
 * Resolves the theme at build time so the first paint does not wait for React or MUI.
 *
 * Emits the application's own stylesheet rather than a parallel vocabulary of its own, so a level that exists in React
 * cannot fail to exist here. Three things land: MUI's generated palette sheets, the design tokens from
 * `getThemeVariables`, and the wash, which is the one value that differs per colour scheme and so cannot be a single
 * declaration.
 */
export function getPreloadThemeCss(): string {
  const theme: Theme = createApplicationTheme();
  const sheets: Array<Record<string, unknown>> = theme.generateStyleSheets();

  return [
    ...sheets.map((sheet: Record<string, unknown>) => toCssRules(sheet)),
    `html {
      ${toDeclarations(getThemeVariables(theme))}
      --xrf-wash: ${getWashImage("dark")};
      color-scheme: dark;
    }`,
    `html[data-color-scheme="light"] {
      --xrf-wash: ${getWashImage("light")};
      color-scheme: light;
    }`,
  ].join("\n");
}
