import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_WATER_SETTINGS } from "@/core/render/lib/features/world/render-water-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The level's water: whether it is drawn as the engine draws it, softened in the shallows, and distorting. */
export function SettingsRendererWater(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("water");

  return (
    <DetailSection title={"Water"} description={"Water surfaces as the engine renders them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_WATER_SETTINGS}
          names={["isEnabled", "isSoft", "isDistorted"]}
          group={features.water}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
