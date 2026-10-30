import { useInjection } from "@wirestate/react";
import { ERenderTerrainTiling, RenderTerrainTiling } from "@xrf/ipc/types/xrf-renderer";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  explainRenderTerrainTiling,
  formatTerrainRotation,
  formatTerrainScale,
  formatTerrainSharpness,
  RENDER_TERRAIN_LIMITS,
  RENDER_TERRAIN_TILING_OPTIONS,
} from "@/core/render/lib/features/render-terrain-choices";
import { TRenderTerrainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** How the terrain lays its detail textures over the ground: repeating as the engines do, or varied by hex tiling. */
export function SettingsRendererTerrain(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const onSet = useRendererOverride("terrain");
  const terrain: TRenderTerrainSettings = renderSettingsService.rendererFeatures.terrain;
  const isEngine: boolean = terrain.tiling === ERenderTerrainTiling.ENGINE;

  return (
    <DetailSection title={"Terrain"} description={"How the ground's detail textures repeat."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Tiling"}
          description={explainRenderTerrainTiling(terrain.tiling)}
          options={RENDER_TERRAIN_TILING_OPTIONS}
          value={terrain.tiling}
          onChange={(tiling: RenderTerrainTiling) => onSet({ tiling })}
        />

        <SliderFormRow
          label={"Scale"}
          description={
            "How small the hexagons are: larger scatters fine repeated features such as rows of pebbles, smaller " +
            "keeps more of each detail whole."
          }
          value={terrain.scale}
          {...RENDER_TERRAIN_LIMITS.scale}
          isDisabled={isEngine}
          format={formatTerrainScale}
          onChange={(scale: number) => onSet({ scale })}
        />

        <SliderFormRow
          label={"Rotation"}
          description={
            "How far each hexagon's copy turns. The details carry painted light and shade, which turns with them and " +
            "mottles the ground, so none by default."
          }
          value={terrain.rotation}
          {...RENDER_TERRAIN_LIMITS.rotation}
          isDisabled={isEngine}
          format={formatTerrainRotation}
          onChange={(rotation: number) => onSet({ rotation })}
        />

        <SliderFormRow
          label={"Blend sharpness"}
          description={"How sharply the copies meet: lower blends wider, softer seams; higher keeps each copy whole."}
          value={terrain.sharpness}
          {...RENDER_TERRAIN_LIMITS.sharpness}
          isDisabled={isEngine}
          format={formatTerrainSharpness}
          onChange={(sharpness: number) => onSet({ sharpness })}
        />
      </div>
    </DetailSection>
  );
}
