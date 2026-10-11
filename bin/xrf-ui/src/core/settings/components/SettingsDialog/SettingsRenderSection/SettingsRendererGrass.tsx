import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  formatGrassDensity,
  formatGrassHeight,
  formatGrassRadius,
  fromGrassDensityScale,
  RENDER_GRASS_LIMITS,
  toGrassDensityScale,
} from "@/core/render/lib/features";
import { TRenderGrassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The level's grass: whether it is planted, how densely, how far round the camera, and how tall. */
export function SettingsRendererGrass(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const grass: TRenderGrassSettings = settingsService.rendererFeatures.grass;

  const onSet = useRendererOverride("grass");

  return (
    <DetailSection title={"Grass"} description={"Detail objects around the camera. The game draws them within 49 m."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Grass"}
          description={"Off, terrain is drawn bare."}
          isChecked={grass.isEnabled}
          onChange={(isEnabled: boolean) => onSet({ isEnabled })}
        />

        <SliderFormRow
          label={"Density"}
          description={"Multiplier over the game's density. Higher costs more to generate and draw."}
          value={toGrassDensityScale(grass.density)}
          {...RENDER_GRASS_LIMITS.density}
          format={(scale: number) => formatGrassDensity(fromGrassDensityScale(scale))}
          onChange={(scale: number) => onSet({ density: fromGrassDensityScale(scale) })}
        />

        <SliderFormRow
          label={"Radius"}
          description={"Draw distance in metres, fading at the edge."}
          value={grass.radius}
          {...RENDER_GRASS_LIMITS.radius}
          format={formatGrassRadius}
          onChange={(radius: number) => onSet({ radius })}
        />

        <SliderFormRow
          label={"Height"}
          description={"Height multiplier."}
          value={grass.height}
          {...RENDER_GRASS_LIMITS.height}
          format={formatGrassHeight}
          onChange={(height: number) => onSet({ height })}
        />

        <SliderFormRow
          label={"Slope limit"}
          description={"Degrees of slope past which no grass grows, thinning out over the last fifth; 90 for none."}
          value={grass.slopeLimit}
          min={10}
          max={90}
          step={1}
          format={(value: number) => `${value.toFixed(0)}°`}
          onChange={(slopeLimit: number) => onSet({ slopeLimit })}
        />

        <CheckboxFormRow
          label={"Aligned to the ground"}
          description={"Stands each tuft along its ground's slope rather than straight up."}
          isChecked={grass.isAligned}
          onChange={(isAligned: boolean) => onSet({ isAligned })}
        />
      </div>
    </DetailSection>
  );
}
