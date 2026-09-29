import { default as FoggyIcon } from "@mui/icons-material/Foggy";
import { ReactElement } from "react";

import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherTriple } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { IRenderChoiceOption } from "@/core/render/lib/features";
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
  onHazed: (isHazed: boolean) => void;
}

/**
 * The fog closing the level in, where the view ends, and what the distance fades into; whether it is drawn.
 */
export function LevelFogAction({
  "data-testid": dataTestId = "level-fog-action",
  id,
  className,
  isOn,
  manual,
  isHazed,
  onToggle,
  onEdit,
  onHazed,
}: ILevelFogActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Fog"}
      description={isOn ? `Fog total at ${formatNumber(manual.fogDistance, 0)} m` : "Fog off"}
      icon={<FoggyIcon />}
      isOn={isOn}
      toggleLabel={"Draw the fog"}
      onToggle={onToggle}
    >
      <LevelWeatherVectorField
        label={"fog_color"}
        isColor
        value={manual.fogColor}
        onChange={(fogColor) => onEdit({ fogColor: toLevelWeatherTriple(fogColor) })}
      />

      <RenderValueSlider
        label={"fog_distance"}
        value={manual.fogDistance}
        {...LEVEL_MANUAL_WEATHER_LIMITS.fogDistance}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onChange={(fogDistance: number) => onEdit({ fogDistance })}
      />

      <RenderValueSlider
        label={"fog_density"}
        value={manual.fogDensity}
        {...LEVEL_MANUAL_WEATHER_LIMITS.fogDensity}
        format={formatPercent}
        onChange={(fogDensity: number) => onEdit({ fogDensity })}
      />

      <RenderValueSlider
        label={"far_plane"}
        value={manual.farPlane}
        {...LEVEL_MANUAL_WEATHER_LIMITS.farPlane}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onChange={(farPlane: number) => onEdit({ farPlane })}
      />

      <RenderValueChoice
        label={"Fades into"}
        options={FADE_OPTIONS}
        value={isHazed ? "haze" : "sky"}
        onChange={(fade: "sky" | "haze") => onHazed(fade === "haze")}
      />

      <LevelWeatherResetButton keys={FOG_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
