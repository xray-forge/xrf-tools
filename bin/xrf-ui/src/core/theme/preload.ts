import { getApplicationBackgroundImage } from "./application-background-image";
import { ACCENT, APPLICATION_BACKGROUND, DIVIDER, LAYOUT, SURFACE } from "./tokens";

/** Resolves theme tokens at build time so the first paint does not wait for React or MUI. */
export function getPreloadThemeCss(): string {
  return [
    `html {
      --preload-title-bar-height: ${LAYOUT.titleBarHeight}px;
      --preload-status-bar-height: ${LAYOUT.statusBarHeight}px;
      --preload-rail-width: ${LAYOUT.railWidth}px;
    }`,
    ...(["dark", "light"] as const).map((scheme) => {
      const selector: string = scheme === "dark" ? "html" : 'html[data-color-scheme="light"]';
      const background: string = getApplicationBackgroundImage({
        primary: ACCENT.primary.main[scheme],
        secondary: ACCENT.secondary.main[scheme],
        primaryOpacity: APPLICATION_BACKGROUND.primary.opacity[scheme],
        secondaryOpacity: APPLICATION_BACKGROUND.secondary.opacity[scheme],
      });

      return `${selector} {
        --preload-surface: ${SURFACE.default[scheme]};
        --preload-paper: ${SURFACE.paper[scheme]};
        --preload-divider: ${DIVIDER[scheme]};
        --preload-background: ${background};
        color-scheme: ${scheme};
      }`;
    }),
  ].join("\n");
}
