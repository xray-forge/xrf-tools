import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_VOLUMETRIC_SETTINGS } from "@/core/render/lib/features/lighting/render-volumetric-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The light the air scatters: the game's shafts, or the enhanced volume, how strong, how forward and how far. */
export function SettingsRendererVolumetric(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("volumetric");

  return (
    <DetailSection title={"Volumetric light"} description={"Light scattered by the air, from the sun and the lamps."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_VOLUMETRIC_SETTINGS}
          names={["mode", "intensity", "anisotropy", "distance"]}
          group={features.volumetric}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
