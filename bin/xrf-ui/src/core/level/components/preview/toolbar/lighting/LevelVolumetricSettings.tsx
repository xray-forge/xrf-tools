import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ERenderVolumetricMode, RenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderVolumetricMode,
  formatVolumetricDistance,
  formatVolumetricStrength,
  RENDER_VOLUMETRIC_LIMITS,
  RENDER_VOLUMETRIC_MODE_OPTIONS,
} from "@/core/render/lib/features/render-volumetric-choices";
import { TRenderVolumetricSettings } from "@/core/render/lib/settings/render-feature-settings";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * How the light the air scatters is drawn: the engine's shafts, or the enhanced froxel volume; and how strongly, how
 * far forward and how far along the view the enhanced volume scatters.
 */
export function LevelVolumetricSettings({
  "data-testid": dataTestId = "level-volumetric-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("volumetric");
  const value: TRenderVolumetricSettings = toLevelModeFeature("volumetric", settings, features);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Volumetric light"}
        options={RENDER_VOLUMETRIC_MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderVolumetricMode) => set({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderVolumetricMode(value.mode)}</p>

      {value.mode === ERenderVolumetricMode.ENHANCED ? (
        <>
          <RenderValueSlider
            label={"Intensity"}
            value={value.intensity}
            {...RENDER_VOLUMETRIC_LIMITS.intensity}
            format={formatVolumetricStrength}
            onChange={(intensity: number) => set({ intensity })}
          />

          <RenderValueSlider
            label={"Anisotropy"}
            value={value.anisotropy}
            {...RENDER_VOLUMETRIC_LIMITS.anisotropy}
            format={formatVolumetricStrength}
            onChange={(anisotropy: number) => set({ anisotropy })}
          />

          <RenderValueSlider
            label={"Distance"}
            value={value.distance}
            {...RENDER_VOLUMETRIC_LIMITS.distance}
            format={formatVolumetricDistance}
            onChange={(distance: number) => set({ distance })}
          />
        </>
      ) : null}

      <Button aria-label={"Reset the volumetric light to the preset"} size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </div>
  );
}
