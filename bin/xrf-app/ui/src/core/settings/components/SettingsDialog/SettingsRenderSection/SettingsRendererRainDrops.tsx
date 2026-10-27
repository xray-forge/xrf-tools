import { useInjection } from "@wirestate/react";
import { ERenderRainDropsMode } from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@xrf/ui/form/SliderFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import { formatGrassRadius } from "@/core/render/lib/features/render-grass-choices";
import {
  explainRenderRainDropsMode,
  formatRainShare,
  RENDER_RAIN_LIMITS,
} from "@/core/render/lib/features/render-rain-choices";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The falling rain: the game's streaks, or drops bending the scene behind them, and their rings on the water. */
export function SettingsRendererRainDrops(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const rain: TRenderRainSettings = renderSettingsService.rendererFeatures.rain;

  const onSet = useRendererOverride("rain");
  const isRefracted: boolean = rain.drops === ERenderRainDropsMode.ENHANCED;

  return (
    <DetailSection title={"Rain"} description={"How the rain falls."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Refracting drops"}
          description={explainRenderRainDropsMode(rain.drops)}
          isChecked={isRefracted}
          onChange={(isOn: boolean) =>
            onSet({ drops: isOn ? ERenderRainDropsMode.ENHANCED : ERenderRainDropsMode.ENGINE })
          }
        />

        <SliderFormRow
          label={"Drop area"}
          description={
            "Metres across the square about the camera raindrops fall over; past 15 m they thin out with distance."
          }
          value={rain.dropSquare}
          {...RENDER_RAIN_LIMITS.dropSquare}
          isDisabled={!isRefracted}
          format={formatGrassRadius}
          onChange={(dropSquare: number) => onSet({ dropSquare })}
        />

        <SliderFormRow
          label={"Ring distance"}
          description={"Metres out to which raindrops leave rings on water and puddles, with puddles enhanced."}
          value={rain.ringDistance}
          {...RENDER_RAIN_LIMITS.ringDistance}
          isDisabled={!isRefracted}
          format={formatGrassRadius}
          onChange={(ringDistance: number) => onSet({ ringDistance })}
        />

        <SliderFormRow
          label={"Ring strength"}
          description={"How strongly raindrops' rings ripple water and puddles, of their designed strength."}
          value={rain.ringStrength}
          {...RENDER_RAIN_LIMITS.ringStrength}
          isDisabled={!isRefracted}
          format={formatRainShare}
          onChange={(ringStrength: number) => onSet({ ringStrength })}
        />

        <SliderFormRow
          label={"Ring size"}
          description={"How wide raindrops' rings spread and how long their waves are, of their designed size."}
          value={rain.ringSize}
          {...RENDER_RAIN_LIMITS.ringSize}
          isDisabled={!isRefracted}
          format={formatRainShare}
          onChange={(ringSize: number) => onSet({ ringSize })}
        />
      </div>
    </DetailSection>
  );
}
