import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ERenderParallaxMode, RenderParallaxMode } from "@/core/ipc/types/xrf-renderer";
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
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** How the level's surfaces are displaced by their height: the engines' steep parallax, or the enhanced terrain march. */
export function SettingsRendererParallax(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const onSet = useRendererOverride("parallax");
  const parallax: TRenderParallaxSettings = renderSettingsService.rendererFeatures.parallax;
  const isEngine: boolean = parallax.mode === ERenderParallaxMode.ENGINE;

  return (
    <DetailSection title={"Parallax"} description={"Displaces surfaces by their height."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Mode"}
          description={explainRenderParallaxMode(parallax.mode)}
          options={RENDER_PARALLAX_MODE_OPTIONS}
          value={parallax.mode}
          onChange={(mode: RenderParallaxMode) => onSet({ mode })}
        />

        <CheckboxFormRow
          label={"Steep parallax"}
          description={"The engine's steep parallax within 12 m on level surfaces whose texture asks for it."}
          isChecked={parallax.isSteep}
          onChange={(isSteep: boolean) => onSet({ isSteep })}
        />

        <SliderFormRow
          label={"Steps"}
          description={"Steps a terrain march takes at most, looking along the ground; three looking straight down."}
          value={parallax.steps}
          {...RENDER_PARALLAX_LIMITS.steps}
          isDisabled={isEngine}
          format={formatParallaxSteps}
          onChange={(steps: number) => onSet({ steps })}
        />

        <SliderFormRow
          label={"Range"}
          description={"Metres within which the terrain is marched, faded out over the last fifth."}
          value={parallax.range}
          {...RENDER_PARALLAX_LIMITS.range}
          isDisabled={isEngine}
          format={formatParallaxRange}
          onChange={(range: number) => onSet({ range })}
        />

        <SliderFormRow
          label={"Height"}
          description={"How deep the relief reaches, in the detail's texture coordinates."}
          value={parallax.height}
          {...RENDER_PARALLAX_LIMITS.height}
          isDisabled={isEngine}
          format={formatParallaxHeight}
          onChange={(height: number) => onSet({ height })}
        />

        <SliderFormRow
          label={"Surface height"}
          description={"How deep the relief reaches on level surfaces whose texture asks for parallax."}
          value={parallax.surfaceHeight}
          {...RENDER_PARALLAX_LIMITS.surfaceHeight}
          isDisabled={isEngine}
          format={formatParallaxHeight}
          onChange={(surfaceHeight: number) => onSet({ surfaceHeight })}
        />

        <SliderFormRow
          label={"Surface range"}
          description={"Metres within which those surfaces are marched, faded out over the last fifth."}
          value={parallax.surfaceRange}
          {...RENDER_PARALLAX_LIMITS.surfaceRange}
          isDisabled={isEngine}
          format={formatParallaxRange}
          onChange={(surfaceRange: number) => onSet({ surfaceRange })}
        />

        <CheckboxFormRow
          label={"Refined"}
          description={"Settles each march where it met the ground by halving its last step three times."}
          isChecked={parallax.isRefined}
          isDisabled={isEngine}
          onChange={(isRefined: boolean) => onSet({ isRefined })}
        />

        <SliderFormRow
          label={"Self-shadowing"}
          description={"How much of the sun the relief keeps from its own hollows."}
          value={parallax.shadow}
          {...RENDER_PARALLAX_LIMITS.shadow}
          isDisabled={isEngine}
          format={formatParallaxStrength}
          onChange={(shadow: number) => onSet({ shadow })}
        />

        <SliderFormRow
          label={"Height blending"}
          description={"How far the terrain's layers blend by height, the one standing highest showing through."}
          value={parallax.blending}
          {...RENDER_PARALLAX_LIMITS.blending}
          isDisabled={isEngine}
          format={formatParallaxStrength}
          onChange={(blending: number) => onSet({ blending })}
        />

        <SliderFormRow
          label={"Cavities"}
          description={"How much the low ground of the relief darkens."}
          value={parallax.cavity}
          {...RENDER_PARALLAX_LIMITS.cavity}
          isDisabled={isEngine}
          format={formatParallaxStrength}
          onChange={(cavity: number) => onSet({ cavity })}
        />
      </div>
    </DetailSection>
  );
}
