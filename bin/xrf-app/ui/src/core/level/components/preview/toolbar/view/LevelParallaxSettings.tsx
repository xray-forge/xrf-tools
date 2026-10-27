import { ERenderParallaxMode, RenderParallaxMode } from "@xrf/ipc/types/xrf-renderer";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderParallaxMode,
  formatParallaxHeight,
  formatParallaxRange,
  formatParallaxSteps,
  formatParallaxStrength,
  RENDER_PARALLAX_LIMITS,
  RENDER_PARALLAX_MODE_OPTIONS,
} from "@/core/render/lib/features/render-parallax-choices";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/render-feature-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";

/** How this view's surfaces are displaced by their height: the engine's steep parallax, or the enhanced terrain march. */
export function LevelParallaxSettings({
  "data-testid": dataTestId = "level-parallax-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("parallax");

  const value: TRenderParallaxSettings = toLevelModeFeature("parallax", settings, features);
  const isEnhanced: boolean = value.mode === ERenderParallaxMode.ENHANCED;

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Mode"}
        options={RENDER_PARALLAX_MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderParallaxMode) => set({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderParallaxMode(value.mode)}</p>

      <CheckboxFormRow
        label={"Steep parallax"}
        description={"Draws the engine's parallax on surfaces whose texture asks for it"}
        isChecked={value.isSteep}
        onChange={(isSteep: boolean) => set({ isSteep })}
      />

      <RenderValueSlider
        label={"Steps"}
        value={value.steps}
        {...RENDER_PARALLAX_LIMITS.steps}
        isDisabled={!isEnhanced}
        format={formatParallaxSteps}
        onChange={(steps: number) => set({ steps })}
      />

      <RenderValueSlider
        label={"Range"}
        value={value.range}
        {...RENDER_PARALLAX_LIMITS.range}
        isDisabled={!isEnhanced}
        format={formatParallaxRange}
        onChange={(range: number) => set({ range })}
      />

      <RenderValueSlider
        label={"Height"}
        value={value.height}
        {...RENDER_PARALLAX_LIMITS.height}
        isDisabled={!isEnhanced}
        format={formatParallaxHeight}
        onChange={(height: number) => set({ height })}
      />

      <RenderValueSlider
        label={"Surface height"}
        value={value.surfaceHeight}
        {...RENDER_PARALLAX_LIMITS.surfaceHeight}
        isDisabled={!isEnhanced}
        format={formatParallaxHeight}
        onChange={(surfaceHeight: number) => set({ surfaceHeight })}
      />

      <RenderValueSlider
        label={"Surface range"}
        value={value.surfaceRange}
        {...RENDER_PARALLAX_LIMITS.surfaceRange}
        isDisabled={!isEnhanced}
        format={formatParallaxRange}
        onChange={(surfaceRange: number) => set({ surfaceRange })}
      />

      <CheckboxFormRow
        label={"Refined"}
        isChecked={value.isRefined}
        isDisabled={!isEnhanced}
        onChange={(isRefined: boolean) => set({ isRefined })}
      />

      <RenderValueSlider
        label={"Self-shadowing"}
        value={value.shadow}
        {...RENDER_PARALLAX_LIMITS.shadow}
        isDisabled={!isEnhanced}
        format={formatParallaxStrength}
        onChange={(shadow: number) => set({ shadow })}
      />

      <RenderValueSlider
        label={"Height blending"}
        value={value.blending}
        {...RENDER_PARALLAX_LIMITS.blending}
        isDisabled={!isEnhanced}
        format={formatParallaxStrength}
        onChange={(blending: number) => set({ blending })}
      />

      <RenderValueSlider
        label={"Cavities"}
        value={value.cavity}
        {...RENDER_PARALLAX_LIMITS.cavity}
        isDisabled={!isEnhanced}
        format={formatParallaxStrength}
        onChange={(cavity: number) => set({ cavity })}
      />

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </div>
  );
}
