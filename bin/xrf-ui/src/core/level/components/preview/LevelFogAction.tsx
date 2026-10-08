import { default as FoggyIcon } from "@mui/icons-material/Foggy";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderFogMode, RenderFogMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelFeatureOptions, TLevelFogOptions } from "@/core/level/lib/features";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderFogMode,
  formatFogDensity,
  formatFogHeight,
  formatFogShare,
  IRenderChoiceOption,
  RENDER_FOG_LIMITS,
  RENDER_FOG_MODE_OPTIONS,
} from "@/core/render/lib/features";
import { TRenderFogSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatNumber, formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const FOG_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["fogColor", "fogDistance", "fogDensity", "farPlane"];

/** What the distance fades into: the sky as the engine fades it, or its haze. */
const FADE_OPTIONS: ReadonlyArray<IRenderChoiceOption<"sky" | "haze">> = [
  { label: "Sky (engine)", value: "sky" },
  { label: "Sky haze", value: "haze" },
];

interface ILevelFogActionProps extends ILevelManualWeatherActionProps {
  /** Whether the distance fades into the sky's haze rather than into the sky itself. */
  isHazed: boolean;
  /** How the view draws the fog beyond the weather's keys: the engine's, or the enhanced fog and its strengths. */
  value: TRenderFogSettings;
  /** What the view sets over the settings, of which the fog's part is changed. */
  features: ILevelFeatureOptions;
  onHazed: (isHazed: boolean) => void;
  onChangeFeatures: (features: ILevelFeatureOptions) => void;
}

/**
 * The fog closing the level in, where the view ends, and what the distance fades into; whether it is drawn, and
 * whether as the engine draws it or as the enhanced fog, thickened low down and scattering the frame's light.
 */
export function LevelFogAction({
  "data-testid": dataTestId = "level-fog-action",
  id,
  className,
  isOn,
  manual,
  isHazed,
  value,
  features,
  onToggle,
  onEdit,
  onHazed,
  onChangeFeatures,
}: ILevelFogActionProps): ReactElement {
  const isEnhanced: boolean = value.mode === ERenderFogMode.ENHANCED;

  const set = useCallback(
    (part: Partial<TLevelFogOptions>): void => onChangeFeatures({ ...features, fog: { ...features.fog, ...part } }),
    [features, onChangeFeatures]
  );

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
      onToggle={onToggle}
    >
      <LevelManualWeatherVectorField field={"fogColor"} isColor manual={manual} onEdit={onEdit} />

      <LevelManualWeatherSlider
        field={"fogDistance"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onEdit={onEdit}
      />

      <LevelManualWeatherSlider field={"fogDensity"} manual={manual} format={formatPercent} onEdit={onEdit} />

      <LevelManualWeatherSlider
        field={"farPlane"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onEdit={onEdit}
      />

      <RenderValueChoice
        label={"Fades into"}
        options={FADE_OPTIONS}
        value={isHazed ? "haze" : "sky"}
        onChange={(fade: "sky" | "haze") => onHazed(fade === "haze")}
      />

      <LevelWeatherResetButton keys={FOG_KEYS} onEdit={onEdit} />

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
        </>
      ) : null}

      <Button size={"small"} onClick={() => onChangeFeatures({ ...features, fog: {} })}>
        Back to the settings for the fog
      </Button>
    </EditorPopoverToggle>
  );
}
