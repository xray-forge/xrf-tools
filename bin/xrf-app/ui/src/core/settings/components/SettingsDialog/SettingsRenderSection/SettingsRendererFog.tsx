import { useInjection } from "@wirestate/react";
import { ERenderFogMode, RenderFogColour, RenderFogMode } from "@xrf/ipc/types/xrf-renderer";
import { ChoiceFormRow } from "@xrf/ui/form/choice-row";
import { SliderFormRow } from "@xrf/ui/form/slider-row";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { ReactElement } from "react";

import {
  explainRenderFogColour,
  explainRenderFogMode,
  formatFogDensity,
  formatFogHeight,
  formatFogShare,
  formatFogSkyFade,
  RENDER_FOG_COLOUR_OPTIONS,
  RENDER_FOG_LIMITS,
  RENDER_FOG_MODE_OPTIONS,
} from "@/core/render/lib/features/render-fog-choices";
import { TRenderFogSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** How the fog is drawn beyond the weather's distance and colour: the game's, or the enhanced fog and its strengths. */
export function SettingsRendererFog(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const fog: TRenderFogSettings = renderSettingsService.rendererFeatures.fog;
  const isEngine: boolean = fog.mode === ERenderFogMode.ENGINE;

  const onSet = useRendererOverride("fog");

  return (
    <DetailSection title={"Fog"} description={"How the weather's fog is drawn: the game's, or thickened low down."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Mode"}
          description={explainRenderFogMode(fog.mode)}
          options={RENDER_FOG_MODE_OPTIONS}
          value={fog.mode}
          onChange={(mode: RenderFogMode) => onSet({ mode })}
        />

        <SliderFormRow
          label={"Height"}
          description={"Metres either side of the world's zero height over which the height fog rises."}
          value={fog.height}
          {...RENDER_FOG_LIMITS.height}
          isDisabled={isEngine}
          format={formatFogHeight}
          onChange={(height: number) => onSet({ height })}
        />

        <SliderFormRow
          label={"Density"}
          description={"How much thicker the distance fog grows where the height fog is whole."}
          value={fog.density}
          {...RENDER_FOG_LIMITS.density}
          isDisabled={isEngine}
          format={formatFogDensity}
          onChange={(density: number) => onSet({ density })}
        />

        <SliderFormRow
          label={"Sun colour"}
          description={"How far the height fog takes the sun's colour, looking towards the sun."}
          value={fog.sunColor}
          {...RENDER_FOG_LIMITS.sunColor}
          isDisabled={isEngine}
          format={formatFogShare}
          onChange={(sunColor: number) => onSet({ sunColor })}
        />

        <SliderFormRow
          label={"Scattering"}
          description={"How readily the frame's bright parts blur into the fog."}
          value={fog.scattering}
          {...RENDER_FOG_LIMITS.scattering}
          isDisabled={isEngine}
          format={formatFogShare}
          onChange={(scattering: number) => onSet({ scattering })}
        />

        <SliderFormRow
          label={"Sky fade from"}
          description={
            "The fog's share past which fogged surfaces fade into the sky; Engine fades them from the first."
          }
          value={fog.skyFade}
          {...RENDER_FOG_LIMITS.skyFade}
          isDisabled={isEngine}
          format={formatFogSkyFade}
          onChange={(skyFade: number) => onSet({ skyFade })}
        />

        <ChoiceFormRow
          label={"Colour"}
          description={explainRenderFogColour(fog.colour)}
          options={RENDER_FOG_COLOUR_OPTIONS}
          value={fog.colour}
          onChange={(colour: RenderFogColour) => onSet({ colour })}
        />
      </div>
    </DetailSection>
  );
}
