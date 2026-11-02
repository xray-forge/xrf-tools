import { useInjection } from "@wirestate/react";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import { RenderSettingRows } from "@/core/render/components/settings/RenderSettingRows";
import {
  formatGrassDensity,
  fromGrassDensityScale,
  RENDER_GRASS_DENSITY_LIMITS,
  RENDER_GRASS_SETTINGS,
  toGrassDensityScale,
} from "@/core/render/lib/features/world/render-grass-choices";
import {
  IRenderFeatureSettings,
  TRenderGrassSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The level's grass: whether it is planted, how densely, how far round the camera, and how tall. */
export function SettingsRendererGrass(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const grass: TRenderGrassSettings = features.grass;

  const onSet = useRendererOverride("grass");

  return (
    <DetailSection title={"Grass"} description={"Detail objects around the camera. The game draws them within 49 m."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <RenderSettingRows
          settings={RENDER_GRASS_SETTINGS}
          names={["isEnabled"]}
          group={grass}
          features={features}
          onSet={onSet}
        />

        <SliderFormRow
          label={RENDER_GRASS_SETTINGS.density.label}
          description={RENDER_GRASS_SETTINGS.density.description as string}
          value={toGrassDensityScale(grass.density)}
          {...RENDER_GRASS_DENSITY_LIMITS}
          format={(scale: number) => formatGrassDensity(fromGrassDensityScale(scale))}
          onChange={(scale: number) => onSet({ density: fromGrassDensityScale(scale) })}
        />

        <RenderSettingRows
          settings={RENDER_GRASS_SETTINGS}
          names={["radius", "height", "slopeLimit", "isAligned", "isShadowed", "shadowRadius", "shadowCascades"]}
          group={grass}
          features={features}
          onSet={onSet}
        />
      </div>
    </DetailSection>
  );
}
