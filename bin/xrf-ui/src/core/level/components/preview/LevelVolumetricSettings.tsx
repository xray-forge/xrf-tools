import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderVolumetricMode, RenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureOptions, TLevelVolumetricOptions } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderVolumetricMode,
  formatVolumetricDistance,
  formatVolumetricStrength,
  RENDER_VOLUMETRIC_LIMITS,
  RENDER_VOLUMETRIC_MODE_OPTIONS,
} from "@/core/render/lib/features";
import { TRenderVolumetricSettings } from "@/core/render/lib/settings/render-feature-settings";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelVolumetricSettingsProps extends BaseComponentProps {
  /** The volumetric lighting as the view draws it. */
  value: TRenderVolumetricSettings;
  /** What the view sets over the settings, of which the volumetric lighting's part is changed. */
  features: ILevelFeatureOptions;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * How the light the air scatters is drawn: the engine's shafts, or the enhanced froxel volume; and how strongly, how
 * far forward and how far along the view the enhanced volume scatters.
 */
export function LevelVolumetricSettings({
  "data-testid": dataTestId = "level-volumetric-settings",
  id,
  className,
  value,
  features,
  onChange,
}: ILevelVolumetricSettingsProps): ReactElement {
  const set = useCallback(
    (part: Partial<TLevelVolumetricOptions>): void =>
      onChange({ ...features, volumetric: { ...features.volumetric, ...part } }),
    [features, onChange]
  );

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Volumetric"}
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

      <Button size={"small"} onClick={() => onChange({ ...features, volumetric: {} })}>
        Back to the settings for the volumetric lighting
      </Button>
    </div>
  );
}
