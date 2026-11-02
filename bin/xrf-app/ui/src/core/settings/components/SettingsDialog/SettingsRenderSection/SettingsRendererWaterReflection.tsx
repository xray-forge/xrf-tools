import { useInjection } from "@wirestate/react";
import { ERenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_REFLECTION_SETTINGS } from "@/core/render/lib/features/tracing/render-reflection-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** Whether the enhanced water reflects the screen, and how far its reflection marches. */
export function SettingsRendererWaterReflection(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  // The game's water reflects the sky's cube alone, whatever is asked of the screen.
  const isWaterEnhanced: boolean = features.water.mode === ERenderWaterMode.ENHANCED;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection title={"Water reflection"} description={"Rivers and lakes, as the enhanced water draws them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_REFLECTION_SETTINGS}
          names={["isWaterReflected", "waterSteps"]}
          group={{
            ...features.reflections,
            isWaterReflected: isWaterEnhanced && features.reflections.isWaterReflected,
          }}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
