import { default as CloudIcon } from "@mui/icons-material/Cloud";
import { ReactElement } from "react";

import { LevelWeatherTexture } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherTextureField } from "@/core/level/components/weather/LevelWeatherTextureField";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherQuad } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatDegrees } from "@/lib/format/angle";
import { formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const CLOUDS_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["cloudsTexture", "cloudsColor", "cloudsRotation"];

interface ILevelCloudsActionProps extends ILevelManualWeatherActionProps {
  /** Every clouds texture the game's weather names. */
  clouds: ReadonlyArray<LevelWeatherTexture>;
}

/**
 * The clouds over the sky: their texture, their colour with the cover in alpha, and how far they are turned.
 */
export function LevelCloudsAction({
  "data-testid": dataTestId = "level-clouds-action",
  id,
  className,
  isOn,
  manual,
  clouds,
  onToggle,
  onEdit,
}: ILevelCloudsActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Clouds"}
      description={
        isOn && manual.cloudsTexture
          ? `Clouds ${manual.cloudsTexture}, ${formatPercent(manual.cloudsColor[3])} cover`
          : "Clouds off"
      }
      icon={<CloudIcon />}
      isOn={isOn}
      toggleLabel={"Draw the clouds"}
      onToggle={onToggle}
    >
      <LevelWeatherTextureField
        label={"clouds_texture"}
        value={manual.cloudsTexture}
        textures={clouds}
        onChange={(cloudsTexture: string) => onEdit({ cloudsTexture })}
      />

      <LevelWeatherVectorField
        label={"clouds_color"}
        isColor
        value={manual.cloudsColor}
        onChange={(cloudsColor) => onEdit({ cloudsColor: toLevelWeatherQuad(cloudsColor) })}
      />

      <RenderValueSlider
        label={"clouds_rotation"}
        value={manual.cloudsRotation}
        {...LEVEL_MANUAL_WEATHER_LIMITS.cloudsRotation}
        format={formatDegrees}
        onChange={(cloudsRotation: number) => onEdit({ cloudsRotation })}
      />

      <LevelWeatherResetButton keys={CLOUDS_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
