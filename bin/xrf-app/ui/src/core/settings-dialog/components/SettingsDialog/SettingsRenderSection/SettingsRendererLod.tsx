import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_LOD_SETTINGS } from "@/core/render/lib/features/world/render-culling-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** When a clump of trees draws as its impostor, and a progressive tree at less than its whole detail. */
export function SettingsRendererLod(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const onSet = useRendererOverride("lod");
  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  return (
    <DetailSection
      title={"Levels of detail"}
      description={"Engine thresholds by screen coverage. The level toolbar can offset impostor distance per view."}
    >
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_LOD_SETTINGS}
          names={["isImpostors", "geometryLod", "ssaA", "ssaB", "ssaDiscard", "ssaGlodStart", "ssaGlodEnd"]}
          group={features.lod}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
