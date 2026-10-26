import { Button } from "@mui/material";
import { ERenderGlassMode, RenderGlassMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderGlassMode,
  formatGlassStrength,
  RENDER_GLASS_LIMITS,
  RENDER_GLASS_MODE_OPTIONS,
} from "@/core/render/lib/features/render-glass-choices";
import { TRenderGlassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * How glass is drawn: the engine's, blended over what is behind it, or the enhanced glass refracting it; and how far
 * the enhanced glass bends it.
 */
export function LevelGlassSettings({
  "data-testid": dataTestId = "level-glass-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("glass");

  const value: TRenderGlassSettings = toLevelModeFeature("glass", settings, features);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Glass mode"}
        options={RENDER_GLASS_MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderGlassMode) => set({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderGlassMode(value.mode)}</p>

      {value.mode === ERenderGlassMode.ENHANCED ? (
        <RenderValueSlider
          label={"Refraction"}
          value={value.refraction}
          {...RENDER_GLASS_LIMITS.refraction}
          format={formatGlassStrength}
          onChange={(refraction: number) => set({ refraction })}
        />
      ) : null}

      <Button size={"small"} aria-label={"Reset the glass to the preset"} onClick={reset}>
        Reset to the preset
      </Button>
    </div>
  );
}
