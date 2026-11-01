import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_CULLING_SETTINGS } from "@/core/render/lib/features/world/render-culling-choices";
import {
  IRenderFeatureSettings,
  TRenderCullingSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/** What the renderer leaves out of a frame because none of it would show. */
export function SettingsRendererCulling(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  return (
    <DetailSection title={"Culling"} description={"Skips geometry that cannot be seen."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_CULLING_SETTINGS}
          names={["isOcclusionCulled", "isDepthPrepassed"]}
          group={features}
          features={features}
          onSet={(part: Partial<TRenderCullingSettings>) => renderSettingsService.setRendererOverrides(part)}
        />
      </div>
    </DetailSection>
  );
}
