import { Button } from "@mui/material";
import { formatCount } from "@xrf/format";
import { ERenderIndirectLightMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { formatOcclusionRadius } from "@/core/render/lib/features/render-ambient-occlusion-choices";
import {
  explainRenderIndirectLightMode,
  formatIndirectLightIntensity,
  RENDER_INDIRECT_LIGHT_LIMITS,
} from "@/core/render/lib/features/render-indirect-light-choices";
import { TRenderIndirectLightSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * The occlusion popover's indirect light: the engine's own, or the light the frame's surfaces bounce onto each other,
 * gathered by the occlusion's visibility bitmask, and how much of it is added.
 */
export function LevelIndirectLightSection({
  "data-testid": dataTestId = "level-indirect-light-section",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("indirectLight");

  const value: TRenderIndirectLightSettings = toLevelModeFeature("indirectLight", settings, features);
  const isEnhanced: boolean = value.mode === ERenderIndirectLightMode.ENHANCED;

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

          <RenderValueSlider
            label={"Bounce steps"}
            value={value.steps}
            {...RENDER_INDIRECT_LIGHT_LIMITS.steps}
            format={formatCount}
            onChange={(steps: number) => set({ steps })}
          />
        </>
      ) : null}

      <Button size={"small"} aria-label={"Reset the indirect light to the preset"} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverGroupSection>
  );
}
