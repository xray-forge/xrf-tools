import { ERenderDebandingMode, RenderDebandingMode } from "@xrf/ipc/types/xrf-renderer";

import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

const RENDER_DEBANDING_MODE_DESCRIPTIONS: Readonly<Record<RenderDebandingMode, string>> = {
  [ERenderDebandingMode.ENGINE]: "Draws the sky as the game does.",
  [ERenderDebandingMode.ENHANCED]:
    "Smooths the sky's colour bands, averaging each texel of its textures with neighbours less than a band apart.",
};

/**
 * @param mode - A sky debanding mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderDebandingMode(mode: RenderDebandingMode): string {
  return RENDER_DEBANDING_MODE_DESCRIPTIONS[mode];
}

/** How each sky debanding setting is set, in the settings and the Look popover alike. */
export const RENDER_DEBANDING_SETTINGS: TRenderSettings<TRenderDebandingSettings> = {
  // Set as a toggle for the enhanced mode, which says what the mode drawn does.
  mode: {
    control: ERenderSettingControl.CUSTOM,
    label: "Debanding",
  },
};
