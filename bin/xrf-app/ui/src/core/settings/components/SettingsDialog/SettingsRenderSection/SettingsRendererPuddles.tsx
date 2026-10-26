import { useInjection } from "@wirestate/react";
import { ERenderRainMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import { formatGrassRadius } from "@/core/render/lib/features/render-grass-choices";
import {
  explainRenderRainMode,
  formatRainPace,
  formatRainShare,
  RENDER_RAIN_LIMITS,
} from "@/core/render/lib/features/render-rain-choices";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** The rain's puddles: whether they gather, how many, how far they reflect and ripple, and whether they fill relief. */
export function SettingsRendererPuddles(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const rain: TRenderRainSettings = renderSettingsService.rendererFeatures.rain;
  const isEnhanced: boolean = rain.mode === ERenderRainMode.ENHANCED;

  const onSet = useRendererOverride("rain");

  return (
    <DetailSection title={"Puddles"} description={"Water gathering on flat ground while it rains."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Puddles"}
          description={explainRenderRainMode(rain.mode)}
          isChecked={isEnhanced}
          onChange={(isOn: boolean) => onSet({ mode: isOn ? ERenderRainMode.ENHANCED : ERenderRainMode.ENGINE })}
        />

        <SliderFormRow
          label={"Amount"}
          description={"How many of the level's puddle sites hold water: about seven in ten at 100%."}
          value={rain.puddles}
          {...RENDER_RAIN_LIMITS.puddles}
          isDisabled={!isEnhanced}
          format={formatRainShare}
          onChange={(puddles: number) => onSet({ puddles })}
        />

        <SliderFormRow
          label={"Reflectivity"}
          description={"How much of a reflection a puddle takes at most."}
          value={rain.reflectivity}
          {...RENDER_RAIN_LIMITS.reflectivity}
          isDisabled={!isEnhanced}
          format={formatRainShare}
          onChange={(reflectivity: number) => onSet({ reflectivity })}
        />

        <SliderFormRow
          label={"Distance"}
          description={"Metres out to which puddles are drawn, fading over the last sixth."}
          value={rain.distance}
          {...RENDER_RAIN_LIMITS.distance}
          isDisabled={!isEnhanced}
          format={formatGrassRadius}
          onChange={(distance: number) => onSet({ distance })}
        />

        <SliderFormRow
          label={"Ripples"}
          description={"How strongly raindrops ripple wet surfaces and puddles."}
          value={rain.ripples}
          {...RENDER_RAIN_LIMITS.ripples}
          isDisabled={!isEnhanced}
          format={formatRainShare}
          onChange={(ripples: number) => onSet({ ripples })}
        />

        <SliderFormRow
          label={"Wetting pace"}
          description={"How many times the game's own pace the level wets, dries, and fills and drains its puddles at."}
          value={rain.pace}
          {...RENDER_RAIN_LIMITS.pace}
          format={formatRainPace}
          onChange={(pace: number) => onSet({ pace })}
        />

        <CheckboxFormRow
          label={"Fill the relief"}
          description={"Water fills the ground's hollows first where the enhanced parallax marches it."}
          isChecked={rain.isReliefFlooded}
          isDisabled={!isEnhanced}
          onChange={(isReliefFlooded: boolean) => onSet({ isReliefFlooded })}
        />
      </div>
    </DetailSection>
  );
}
