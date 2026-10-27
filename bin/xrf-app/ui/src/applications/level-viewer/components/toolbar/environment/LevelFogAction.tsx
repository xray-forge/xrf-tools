import { useInjection } from "@wirestate/react";
import { formatNumber, formatPercent } from "@xrf/format";
import { ERenderFogMode, RenderFogMode } from "@xrf/ipc/types/xrf-renderer";
import { EditorPopoverToggle } from "@xrf/ui/editor/popover-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";
import { FoggyIcon } from "@xrf/ui/icons";
import { Button } from "@xrf/ui/material";
import { ReactElement } from "react";

import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_FOG_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderFogMode,
  formatFogDensity,
  formatFogHeight,
  formatFogShare,
  formatFogSkyFade,
  RENDER_FOG_LIMITS,
  RENDER_FOG_MODE_OPTIONS,
} from "@/core/render/lib/features/render-fog-choices";
import { TRenderFogSettings } from "@/core/render/lib/settings/render-feature-settings";

/** What the distance fades into: the sky as the engine fades it, or its haze. */
const FADE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<"sky" | "haze">> = [
  { label: "Sky (engine)", value: "sky" },
  { label: "Sky haze", value: "haze" },
];

/**
 * The fog closing the level in, where the view ends, and what the distance fades into; whether it is drawn, and
 * whether as the engine draws it or as the enhanced fog, thickened low down and scattering the frame's light.
 */
export function LevelFogAction({
  "data-testid": dataTestId = "level-fog-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("fog");
  const { isOn, toggle } = useLevelViewToggle("isFogged");

  const manual: ILevelManualWeather = weatherService.shown;
  // Whether the distance fades into the sky's haze rather than into the sky itself.
  const isHazed: boolean = viewService.options.isSkyHazed;
  // How the view draws the fog beyond the weather's keys: the engine's, or the enhanced fog and its strengths.
  const value: TRenderFogSettings = toLevelModeFeature("fog", settings, features);
  const isEnhanced: boolean = value.mode === ERenderFogMode.ENHANCED;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Fog"}
      description={
        isOn
          ? `Fog total at ${formatNumber(manual.fogDistance, 0)} m` + (isEnhanced ? ", thickened low down" : "")
          : "Fog off"
      }
      icon={<FoggyIcon />}
      isOn={isOn}
      toggleLabel={"Fog"}
      onToggle={toggle}
    >
      <LevelManualWeatherVectorField field={"fogColor"} isColor manual={manual} onEdit={weatherService.editManual} />

      <LevelManualWeatherSlider
        field={"fogDistance"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"fogDensity"}
        manual={manual}
        format={formatPercent}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"farPlane"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onEdit={weatherService.editManual}
      />

      <RenderValueChoice
        label={"Fades into"}
        options={FADE_OPTIONS}
        value={isHazed ? "haze" : "sky"}
        onChange={(fade: "sky" | "haze") => viewService.setOption("isSkyHazed", fade === "haze")}
      />

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_FOG_KEYS} onReset={weatherService.resetManual} />

      <RenderValueChoice
        label={"Mode"}
        options={RENDER_FOG_MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderFogMode) => set({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderFogMode(value.mode)}</p>

      <RenderValueSlider
        label={"Height"}
        value={value.height}
        {...RENDER_FOG_LIMITS.height}
        isDisabled={!isEnhanced}
        format={formatFogHeight}
        onChange={(height: number) => set({ height })}
      />

      <RenderValueSlider
        label={"Density"}
        value={value.density}
        {...RENDER_FOG_LIMITS.density}
        isDisabled={!isEnhanced}
        format={formatFogDensity}
        onChange={(density: number) => set({ density })}
      />

      <RenderValueSlider
        label={"Sun colour"}
        value={value.sunColor}
        {...RENDER_FOG_LIMITS.sunColor}
        isDisabled={!isEnhanced}
        format={formatFogShare}
        onChange={(sunColor: number) => set({ sunColor })}
      />

      <RenderValueSlider
        label={"Scattering"}
        value={value.scattering}
        {...RENDER_FOG_LIMITS.scattering}
        isDisabled={!isEnhanced}
        format={formatFogShare}
        onChange={(scattering: number) => set({ scattering })}
      />

      <RenderValueSlider
        label={"Sky fade from"}
        value={value.skyFade}
        {...RENDER_FOG_LIMITS.skyFade}
        isDisabled={!isEnhanced}
        format={formatFogSkyFade}
        onChange={(skyFade: number) => set({ skyFade })}
      />

      <Button size={"small"} onClick={reset}>
        Reset to the preset
      </Button>
    </EditorPopoverToggle>
  );
}
