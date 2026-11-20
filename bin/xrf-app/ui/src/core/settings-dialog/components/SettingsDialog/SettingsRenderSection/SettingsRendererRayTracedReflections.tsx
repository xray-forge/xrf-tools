import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_REFLECTION_SETTINGS } from "@/core/render/lib/features/tracing/render-reflection-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** Whether what the surfaces' and the water's rays miss on the screen is traced against the level, and how often. */
export function SettingsRendererRayTracedReflections(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection
      title={"Ray-traced reflections"}
      description={"What the surfaces' and the water's rays miss on the screen, traced against the level."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_REFLECTION_SETTINGS}
          names={["isTraced", "traceInterval", "isWaterFullyTraced"]}
          group={features.reflections}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
