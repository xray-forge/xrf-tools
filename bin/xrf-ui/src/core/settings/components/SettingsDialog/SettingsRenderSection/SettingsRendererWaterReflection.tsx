import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ERenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import { RENDER_REFLECTION_LIMITS } from "@/core/render/lib/features/render-reflection-choices";
import { IRenderFeatureSettings, TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** Whether the enhanced water reflects the screen, and how far its reflection marches. */
export function SettingsRendererWaterReflection(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const reflections: TRenderReflectionSettings = features.reflections;
  const isWaterEnhanced: boolean = features.water.mode === ERenderWaterMode.ENHANCED;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection title={"Water reflection"} description={"Rivers and lakes, as the enhanced water draws them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Water reflects the screen"}
          description={
            isWaterEnhanced
              ? "Off, the water reflects the sky's cube alone and keeps the rest of its look."
              : "The game's water reflects the sky's cube; set the water to Enhanced."
          }
          isChecked={isWaterEnhanced && reflections.isWaterReflected}
          isDisabled={!isWaterEnhanced}
          onChange={(isWaterReflected: boolean) => onSet({ isWaterReflected })}
        />

        <SliderFormRow
          label={"March steps"}
          description={"How many steps the water's reflection marches over the screen."}
          value={reflections.waterSteps}
          {...RENDER_REFLECTION_LIMITS.waterSteps}
          isDisabled={!isWaterEnhanced || !reflections.isWaterReflected}
          format={formatCount}
          onChange={(waterSteps: number) => onSet({ waterSteps })}
        />
      </div>
    </DetailSection>
  );
}
