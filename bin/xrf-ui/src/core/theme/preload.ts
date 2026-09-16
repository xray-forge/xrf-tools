import { Theme } from "@mui/material";

import { createApplicationTheme } from "./creation";
import { getWashImage } from "./surface";
import { BADGE_FONT_SIZE, CODE, CONTENT_STATE, DIALOG, LAYOUT, MONOSPACE, PANEL, RADIUS, TREE } from "./tokens";

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
  const theme: Theme = createApplicationTheme();
  const sheets: Array<Record<string, unknown>> = theme.generateStyleSheets();

  return [
    ...sheets.map((sheet: Record<string, unknown>) => toCssRules(sheet)),
    `html {
      --xrf-tree-row-height: ${TREE.rowHeight}px;
      --xrf-tree-icon-width: ${TREE.iconWidth}px;
      --xrf-tree-icon-size: ${TREE.iconSize}px;
      --xrf-tree-icon-gap: ${TREE.iconGap}px;
      --xrf-code-line-height: ${CODE.lineHeight}px;
      --xrf-header-height: ${LAYOUT.headerHeight}px;
      --xrf-radius-surface: ${RADIUS.md}px;
      --xrf-reading-column: ${LAYOUT.readingColumnWidth}px;
      --xrf-reading-label: ${LAYOUT.readingLabelWidth}px;
      --xrf-dialog-padding-x: ${DIALOG.paddingX * 8}px;
      --xrf-panel-line-height: ${PANEL.contentLineHeight};
      --xrf-panel-property-padding: ${PANEL.propertyPaddingY * 8}px;
      --xrf-monospace-family: ${MONOSPACE.fontFamily};
      --xrf-content-state-icon: ${CONTENT_STATE.iconSize}px;
      --xrf-badge-font-size: ${BADGE_FONT_SIZE};
      --xrf-body2-tracking: ${theme.typography.body2.letterSpacing};
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
