import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ERenderParallaxMode, RenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatParallaxHeight,
  formatParallaxRange,
  formatParallaxSteps,
  formatParallaxStrength,
  RENDER_PARALLAX_LIMITS,
  RENDER_PARALLAX_MODE_OPTIONS,
} from "@/core/render/lib/features/render-parallax-choices";
import { TRenderParallaxSettings } from "@/core/render/lib/settings/render-feature-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** How this view's surfaces are displaced by their height: the engine's steep parallax, or the enhanced terrain march. */
export function LevelParallaxSettings({
  "data-testid": dataTestId = "level-parallax-settings",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("parallax");

  const value: TRenderParallaxSettings = toLevelModeFeature("parallax", settings, features);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <RenderValueChoice
        label={"Mode"}
        options={RENDER_PARALLAX_MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderParallaxMode) => set({ mode })}
      />

      <CheckboxFormRow
        label={"Steep parallax"}
        description={"The engine's on surfaces whose texture asks for it"}
        isChecked={value.isSteep}
        onChange={(isSteep: boolean) => set({ isSteep })}
      />

      {value.mode === ERenderParallaxMode.ENHANCED ? (
        <>
          <RenderValueSlider
            label={"Steps"}
            value={value.steps}
            {...RENDER_PARALLAX_LIMITS.steps}
            format={formatParallaxSteps}
            onChange={(steps: number) => set({ steps })}
          />

          <RenderValueSlider
            label={"Range"}
            value={value.range}
            {...RENDER_PARALLAX_LIMITS.range}
            format={formatParallaxRange}
            onChange={(range: number) => set({ range })}
          />

          <RenderValueSlider
            label={"Height"}
            value={value.height}
            {...RENDER_PARALLAX_LIMITS.height}
            format={formatParallaxHeight}
            onChange={(height: number) => set({ height })}
          />

          <RenderValueSlider
            label={"Surface height"}
            value={value.surfaceHeight}
            {...RENDER_PARALLAX_LIMITS.surfaceHeight}
            format={formatParallaxHeight}
            onChange={(surfaceHeight: number) => set({ surfaceHeight })}
          />

          <RenderValueSlider
            label={"Surface range"}
            value={value.surfaceRange}
            {...RENDER_PARALLAX_LIMITS.surfaceRange}
            format={formatParallaxRange}
            onChange={(surfaceRange: number) => set({ surfaceRange })}
          />

          <CheckboxFormRow
            label={"Refined"}
            isChecked={value.isRefined}
            onChange={(isRefined: boolean) => set({ isRefined })}
          />

          <RenderValueSlider
            label={"Self-shadowing"}
            value={value.shadow}
            {...RENDER_PARALLAX_LIMITS.shadow}
            format={formatParallaxStrength}
            onChange={(shadow: number) => set({ shadow })}
          />

          <RenderValueSlider
            label={"Height blending"}
            value={value.blending}
            {...RENDER_PARALLAX_LIMITS.blending}
            format={formatParallaxStrength}
            onChange={(blending: number) => set({ blending })}
          />

          <RenderValueSlider
            label={"Cavities"}
            value={value.cavity}
            {...RENDER_PARALLAX_LIMITS.cavity}
            format={formatParallaxStrength}
            onChange={(cavity: number) => set({ cavity })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </div>
  );
}
