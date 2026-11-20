import { COLOR_SCHEME_ATTRIBUTE } from "#/theme";

// The module itself rather than its family's index, which also carries `ThemeRoot` and the stylesheets it imports:
// this runs in Node, ahead of the preview's bundle.
import { getPreloadThemeCss } from "../src/theme/root/preload";
import { COLOR_SCHEME_STORAGE_KEY } from "./color-scheme-storage";

/** The id `preview.tsx` finds this stylesheet by, so a hot update rewrites it rather than adding a second one. */
export const PRELOAD_STYLE_ID: string = "xrf-storybook-preload-theme";

/**
 * Sets the scheme on the document before anything paints: the toolbar's, from the `globals` the manager hands the
 * iframe, else the one MUI last stored, else dark, as `ThemeRoot` defaults to.
 */
function getSchemeScript(): string {
  return `(() => {
  const element = document.documentElement;
  let scheme = "dark";

  try {
    const globals = new URLSearchParams(window.location.search).get("globals") ?? "";
    const fromUrl = /(?:^|;)scheme:(light|dark)(?:;|$)/.exec(globals)?.[1];
    const stored = window.localStorage.getItem(${JSON.stringify(COLOR_SCHEME_STORAGE_KEY)});

    scheme = fromUrl ?? (stored === "light" || stored === "dark" ? stored : "dark");
  } catch {}

  element.setAttribute(${JSON.stringify(COLOR_SCHEME_ATTRIBUTE)}, scheme);
})();`;
}

/**
 * Storybook's own preview states (preparing a story or docs, no preview, an error) and the bare page under them are
 * white by default. Painted here with the theme's variables instead, so they follow the scheme with everything else.
 */
const PREVIEW_STATE_CSS: string = `
html,
body,
.sb-preparing-story,
.sb-preparing-docs,
.sb-nopreview,
.sb-errordisplay {
  background: var(--mui-palette-background-default);
  color: var(--mui-palette-text-primary);
}

.sb-nopreview_main,
.sb-nopreview_main * {
  background: var(--mui-palette-background-paper);
  color: var(--mui-palette-text-primary);
}

.sb-loader {
  border-color: var(--mui-palette-divider);
  border-top-color: var(--mui-palette-primary-main);
  mix-blend-mode: normal;
}
`;

/**
 * What the preview iframe's head carries ahead of any module: the scheme, the application's first-paint stylesheet,
 * and Storybook's own states painted from it.
 *
 * @returns Markup appended to Storybook's preview head.
 */
export function getPreviewHeadHtml(): string {
  return [
    `<script>${getSchemeScript()}</script>`,
    `<style id="${PRELOAD_STYLE_ID}">${getPreloadThemeCss()}</style>`,
    `<style>${PREVIEW_STATE_CSS}</style>`,
  ].join("\n");
}
