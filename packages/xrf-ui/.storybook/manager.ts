import { parseHexColor } from "@xrf/math";
import { RESULT_WHATS_NEW_DATA } from "storybook/internal/core-events";
import { addons } from "storybook/manager-api";
import { create } from "storybook/theming";

// The token module itself: the manager is bundled apart from the preview and needs nothing of the theme but its values.
import { ACCENT, DIVIDER, MONOSPACE, RADIUS, STATE, STATE_TONE, SURFACE, TEXT } from "../src/theme/tokens";

/**
 * A state scrim as `rgba()`: Storybook derives shades from some theme colours with `polished`, which cannot read the
 * `color-mix()` the kit's own scrims are written in.
 */
function toScrim(tone: string, share: number): string {
  const value: number = parseHexColor(tone);

  return `rgba(${(value >> 16) & 0xff}, ${(value >> 8) & 0xff}, ${value & 0xff}, ${share})`;
}

/**
 * `core.disableWhatsNewNotifications` stops the release-notes notification, but 10.6's settings button still draws an
 * unread dot from a flag nothing writes. Marking each post read once the manager has stored it clears the dot.
 */
addons.register("xrf/quiet-whats-new", (api) => {
  api.on(RESULT_WHATS_NEW_DATA, () => setTimeout(() => api.whatsNewHasBeenRead(), 0));
});

/** The manager drawn as the application's dark frame: the frame around, the reading plane inside, amber for selection. */
addons.setConfig({
  theme: create({
    base: "dark",
    brandTitle: "xrf ui",
    colorPrimary: ACCENT.primary.main.dark,
    colorSecondary: ACCENT.primary.main.dark,
    appBg: SURFACE.frame.dark,
    appContentBg: SURFACE.content.dark,
    appPreviewBg: SURFACE.content.dark,
    appHoverBg: toScrim(STATE_TONE.dark, STATE.hover.dark),
    appBorderColor: DIVIDER.dark,
    appBorderRadius: RADIUS.md,
    fontCode: MONOSPACE.fontFamily,
    textColor: TEXT.primary.dark,
    textMutedColor: TEXT.secondary.dark,
    textInverseColor: SURFACE.content.dark,
    barBg: SURFACE.frame.dark,
    barTextColor: TEXT.secondary.dark,
    barHoverColor: ACCENT.primary.main.dark,
    barSelectedColor: ACCENT.primary.main.dark,
    buttonBg: SURFACE.overlay.dark,
    buttonBorder: DIVIDER.dark,
    booleanBg: SURFACE.content.dark,
    booleanSelectedBg: SURFACE.overlay.dark,
    inputBg: SURFACE.content.dark,
    inputBorder: DIVIDER.dark,
    inputTextColor: TEXT.primary.dark,
    inputBorderRadius: RADIUS.sm,
  }),
});
