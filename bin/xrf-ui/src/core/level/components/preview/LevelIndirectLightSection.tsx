import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderIndirectLightMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureOptions, TLevelIndirectLightOptions } from "@/core/level/lib/features";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderIndirectLightMode,
  formatIndirectLightIntensity,
  formatOcclusionRadius,
  RENDER_INDIRECT_LIGHT_LIMITS,
} from "@/core/render/lib/features";
import { TRenderIndirectLightSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelIndirectLightSectionProps extends BaseComponentProps {
  /** The indirect light as the view draws it. */
  value: TRenderIndirectLightSettings;
  /** What the view sets over the settings, of which the indirect light's part is changed. */
  features: ILevelFeatureOptions;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * The occlusion popover's indirect light: the engine's own, or the light the frame's surfaces bounce onto each other,
 * gathered by the occlusion's visibility bitmask, and how much of it is added.
 */
export function LevelIndirectLightSection({
  "data-testid": dataTestId = "level-indirect-light-section",
  id,
  className,
  value,
  features,
  onChange,
}: ILevelIndirectLightSectionProps): ReactElement {
  const isEnhanced: boolean = value.mode === ERenderIndirectLightMode.ENHANCED;

  const set = useCallback(
    (part: Partial<TLevelIndirectLightOptions>): void =>
      onChange({ ...features, indirectLight: { ...features.indirectLight, ...part } }),
    [features, onChange]
  );

  return (
    <EditorPopoverGroupSection
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Indirect light"}
      description={explainRenderIndirectLightMode(value.mode)}
      isOn={isEnhanced}
      onToggle={() => set({ mode: isEnhanced ? ERenderIndirectLightMode.ENGINE : ERenderIndirectLightMode.ENHANCED })}
    >
      {isEnhanced ? (
        <>
          <RenderValueSlider
            label={"Intensity"}
            value={value.intensity}
            {...RENDER_INDIRECT_LIGHT_LIMITS.intensity}
            format={formatIndirectLightIntensity}
            onChange={(intensity: number) => set({ intensity })}
          />

          <RenderValueSlider
            label={"Bounce radius"}
            value={value.radius}
            {...RENDER_INDIRECT_LIGHT_LIMITS.radius}
            format={formatOcclusionRadius}
            onChange={(radius: number) => set({ radius })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={() => onChange({ ...features, indirectLight: {} })}>
        Back to the settings for the indirect light
      </Button>
    </EditorPopoverGroupSection>
  );
}
