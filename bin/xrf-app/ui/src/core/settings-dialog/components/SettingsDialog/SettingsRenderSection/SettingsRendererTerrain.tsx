import { useInjection } from "@wirestate/react";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import { RENDER_TERRAIN_SETTINGS } from "@/core/render/lib/features/world/render-terrain-choices";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** How the terrain lays its detail textures over the ground: repeating as the engines do, or varied by hex tiling. */
export function SettingsRendererTerrain(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;

  const onSet = useRendererOverride("terrain");

  return (
    <DetailSection title={"Terrain"} description={"How the ground's detail textures repeat."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_TERRAIN_SETTINGS}
          names={["tiling", "scale", "rotation", "sharpness"]}
          group={features.terrain}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
