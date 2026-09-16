import { Theme } from "@mui/material";

import {
  BADGE_FONT_SIZE,
  CODE,
  CONTENT_STATE,
  DIALOG,
  LAYOUT,
  MONOSPACE,
  PANEL,
  RADIUS,
  TREE,
  VIEWPORT,
} from "./tokens";

/** Every published property, so a consumer and a test name the same thing. */
export type TThemeVariableName = keyof ReturnType<typeof getThemeVariables>;

/** A measurement declared as a number of pixels, which is how `tokens.ts` states every length. */
function px(value: number): string {
  return `${value}px`;
}

/**
 * The values CSS can read, projected from the modules that own them.
 *
 * @param theme - Resolved application theme, for the values MUI owns rather than `tokens.ts`.
 * @returns Custom property names mapped to their CSS values.
 */
export function getThemeVariables(theme: Theme) {
  return {
    "--xrf-badge-font-size": BADGE_FONT_SIZE,
    "--xrf-checkerboard-dark": VIEWPORT.checkerboardDark,
    "--xrf-checkerboard-light": VIEWPORT.checkerboardLight,
    "--xrf-checkerboard-square": px(VIEWPORT.checkerboardSquare),
    "--xrf-code-line-height": px(CODE.lineHeight),
    "--xrf-content-state-icon": px(CONTENT_STATE.iconSize),
    "--xrf-dialog-padding-x": theme.spacing(DIALOG.paddingX),
    "--xrf-header-height": px(LAYOUT.headerHeight),
    "--xrf-monospace-family": MONOSPACE.fontFamily,
    "--xrf-panel-line-height": String(PANEL.contentLineHeight),
    "--xrf-panel-property-padding": theme.spacing(PANEL.propertyPaddingY),
    "--xrf-radius-surface": px(RADIUS.md),
    "--xrf-rail-width": px(LAYOUT.railWidth),
    "--xrf-reading-column": px(LAYOUT.readingColumnWidth),
    "--xrf-reading-label": px(LAYOUT.readingLabelWidth),
    "--xrf-status-bar-height": px(LAYOUT.statusBarHeight),
    "--xrf-surface-radius": px(LAYOUT.surfaceRadius),
    "--xrf-title-bar-height": px(LAYOUT.titleBarHeight),
    "--xrf-tree-icon-gap": px(TREE.iconGap),
    "--xrf-tree-icon-size": px(TREE.iconSize),
    "--xrf-tree-icon-width": px(TREE.iconWidth),
    "--xrf-tree-row-height": px(TREE.rowHeight),
    "--xrf-viewport-backdrop": VIEWPORT.backdrop,
  } as const;
}
