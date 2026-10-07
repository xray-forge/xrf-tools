import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderGlassMode, RenderGlassMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureOptions, TLevelGlassOptions } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderGlassMode,
  formatGlassStrength,
  RENDER_GLASS_LIMITS,
  RENDER_GLASS_MODE_OPTIONS,
} from "@/core/render/lib/features";
import { TRenderGlassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelGlassSettingsProps extends BaseComponentProps {
  /** The glass as the view draws it. */
  value: TRenderGlassSettings;
  /** What the view sets over the settings, of which the glass's part is changed. */
  features: ILevelFeatureOptions;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * How glass is drawn: the engine's, blended over what is behind it, or the enhanced glass refracting it; and how far
 * the enhanced glass bends it.
 */
export function LevelGlassSettings({
  "data-testid": dataTestId = "level-glass-settings",
  id,
  className,
  value,
  features,
  onChange,
}: ILevelGlassSettingsProps): ReactElement {
  const set = useCallback(
    (part: Partial<TLevelGlassOptions>): void => onChange({ ...features, glass: { ...features.glass, ...part } }),
    [features, onChange]
  );

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Glass"}
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

      <Button size={"small"} onClick={() => onChange({ ...features, glass: {} })}>
        Back to the settings for the glass
      </Button>
    </div>
  );
}
