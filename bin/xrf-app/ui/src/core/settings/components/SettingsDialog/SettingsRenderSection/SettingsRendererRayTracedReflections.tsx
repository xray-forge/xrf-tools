import { useInjection } from "@wirestate/react";
import { ERenderAntialiasing } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/checkbox-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  formatReflectionTraceInterval,
  RENDER_REFLECTION_LIMITS,
} from "@/core/render/lib/features/render-reflection-choices";
import { IRenderFeatureSettings, TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** Whether what the surfaces' and the water's rays miss on the screen is traced against the level, and how often. */
export function SettingsRendererRayTracedReflections(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const reflections: TRenderReflectionSettings = features.reflections;
  const isRayReconstructed: boolean =
    features.antialiasing === ERenderAntialiasing.DLSS && features.upscaling.isRayReconstructed;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection
      title={"Ray-traced reflections"}
      description={"What the surfaces' and the water's rays miss on the screen, traced against the level."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Ray-traced reflections"}
          description={
            "Traces what the screen misses against the level's geometry on a GPU that traces rays, for the " +
            "surfaces and the water alike; elsewhere they reflect the sky's cube."
          }
          isChecked={reflections.isTraced}
          onChange={(isTraced: boolean) => onSet({ isTraced })}
        />

        <SliderFormRow
          label={"Re-trace"}
          description={
            isRayReconstructed
              ? "How often a missed pixel traces again, the last answer held between; the surfaces trace every " +
                "frame under ray reconstruction."
              : "How often a missed pixel traces again, the last answer held between."
          }
          value={reflections.traceInterval}
          {...RENDER_REFLECTION_LIMITS.traceInterval}
          isDisabled={!reflections.isTraced}
          format={formatReflectionTraceInterval}
          onChange={(traceInterval: number) => onSet({ traceInterval })}
        />
      </div>
    </DetailSection>
  );
}
