import { default as NightsStayIcon } from "@mui/icons-material/NightsStay";
import { ReactElement } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherTextureField } from "@/core/level/components/weather/LevelWeatherTextureField";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherTriple } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatDegrees } from "@/lib/format/angle";

/** The keys the popover sets. */
const SKY_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["skyTexture", "skyColor", "skyRotation"];

interface ILevelSkyActionProps extends ILevelManualWeatherActionProps {
  /** Every sky the game's weather names. */
  skies: ReadonlyArray<LevelWeatherTexture>;
}

/**
 * The sky cube, its tint and how far it is turned, which its `#small` twin follows; whether it is drawn.
 */
export function LevelSkyAction({
  "data-testid": dataTestId = "level-sky-action",
  id,
  className,
  isOn,
  manual,
  skies,
  onToggle,
  onEdit,
}: ILevelSkyActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sky"}
      description={isOn ? `Sky ${manual.skyTexture || "none"}` : "Sky off, the backdrop behind the level"}
      icon={<NightsStayIcon />}
      isOn={isOn}
      toggleLabel={"Draw the sky"}
      onToggle={onToggle}
    >
      <LevelWeatherTextureField
        label={"sky_texture"}
        value={manual.skyTexture}
        textures={skies}
        onChange={(skyTexture: string) => onEdit({ skyTexture })}
      />

      <LevelWeatherVectorField
        label={"sky_color"}
        isColor
        value={manual.skyColor}
        onChange={(skyColor) => onEdit({ skyColor: toLevelWeatherTriple(skyColor) })}
      />

      <RenderValueSlider
        label={"sky_rotation"}
        value={manual.skyRotation}
        {...LEVEL_MANUAL_WEATHER_LIMITS.skyRotation}
        format={formatDegrees}
        onChange={(skyRotation: number) => onEdit({ skyRotation })}
      />

      <LevelWeatherResetButton keys={SKY_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
