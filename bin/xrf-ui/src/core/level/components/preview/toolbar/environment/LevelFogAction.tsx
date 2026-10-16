import { default as FoggyIcon } from "@mui/icons-material/Foggy";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatNumber, formatPercent } from "@xrf/format";
import { ReactElement } from "react";

import { ERenderFogMode, RenderFogMode } from "@/core/ipc/types/xrf-renderer";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";
import {
  explainRenderFogMode,
  formatFogDensity,
  formatFogHeight,
  formatFogShare,
  RENDER_FOG_LIMITS,
  RENDER_FOG_MODE_OPTIONS,
} from "@/core/render/lib/features/render-fog-choices";
import { TRenderFogSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The keys the popover sets. */
const FOG_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["fogColor", "fogDistance", "fogDensity", "farPlane"];

/** What the distance fades into: the sky as the engine fades it, or its haze. */
const FADE_OPTIONS: ReadonlyArray<IRenderChoiceOption<"sky" | "haze">> = [
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
      toggleLabel={"Draw the fog"}
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

      <LevelWeatherResetButton keys={FOG_KEYS} onEdit={weatherService.editManual} />

      <RenderValueChoice
        label={"Drawn as"}
        options={RENDER_FOG_MODE_OPTIONS}
        value={value.mode}
        onChange={(mode: RenderFogMode) => set({ mode })}
      />

      <p className={"text-xs text-text-secondary"}>{explainRenderFogMode(value.mode)}</p>

      {isEnhanced ? (
        <>
          <RenderValueSlider
            label={"Height"}
            value={value.height}
            {...RENDER_FOG_LIMITS.height}
            format={formatFogHeight}
            onChange={(height: number) => set({ height })}
          />

          <RenderValueSlider
            label={"Density"}
            value={value.density}
            {...RENDER_FOG_LIMITS.density}
            format={formatFogDensity}
            onChange={(density: number) => set({ density })}
          />

          <RenderValueSlider
            label={"Sun colour"}
            value={value.sunColor}
            {...RENDER_FOG_LIMITS.sunColor}
            format={formatFogShare}
            onChange={(sunColor: number) => set({ sunColor })}
          />

          <RenderValueSlider
            label={"Scattering"}
            value={value.scattering}
            {...RENDER_FOG_LIMITS.scattering}
            format={formatFogShare}
            onChange={(scattering: number) => set({ scattering })}
          />

          <RenderValueSlider
            label={"Sky fade from"}
            value={value.skyFade}
            {...RENDER_FOG_LIMITS.skyFade}
            format={(share: number) => (share > 0 ? `${Math.round(share * 100)}% fog` : "Engine")}
            onChange={(skyFade: number) => set({ skyFade })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={reset}>
        Back to the settings for the fog
      </Button>
    </EditorPopoverToggle>
  );
}
