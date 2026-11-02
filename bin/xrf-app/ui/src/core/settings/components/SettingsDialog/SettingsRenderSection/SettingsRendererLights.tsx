import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_LIGHTS_SETTINGS } from "@/core/render/lib/features/lighting/render-lights-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/**
 * The local lights: whether they light the level, whether the level file's own join the spawned ones, and which draw
 * shafts how strongly.
 */
export function SettingsRendererLights(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("lights");

  return (
    <DetailSection title={"Lamps"} description={"The level's point and spot lamps."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_LIGHTS_SETTINGS}
          names={[
            "isEnabled",
            "isShadowed",
            "shadowFilter",
            "falloff",
            "isVolumetric",
            "shafts",
            "shaftStrength",
            "isLevelLights",
          ]}
          group={features.lights}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
