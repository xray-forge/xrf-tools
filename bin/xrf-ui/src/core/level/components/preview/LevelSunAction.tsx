import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { Button, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather, toLevelManualSun } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherQuad, toLevelWeatherTriple } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatDegrees } from "@/lib/format/angle";

/** The keys the popover sets. */
const SUN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "sunColor",
  "sunAltitude",
  "sunLongitude",
  "ambientColor",
  "hemisphereColor",
];

interface ILevelSunActionProps extends ILevelManualWeatherActionProps {
  /** The sun the open level was compiled against, or null where it names none. */
  sun: Nullable<LevelSunDescription>;
}

/**
 * The sun, the ambient and the hemisphere, by the keys a weather writes them with; whether the sun is drawn in the sky.
 */
export function LevelSunAction({
  "data-testid": dataTestId = "level-sun-action",
  id,
  className,
  isOn,
  manual,
  sun,
  onToggle,
  onEdit,
}: ILevelSunActionProps): ReactElement {
  const compiled = useMemo(() => toLevelManualSun(sun?.direction ?? null), [sun]);

  // The occlusion a level carries was computed for its sun, so lighting it from there is lighting it as xrLC assumed.
  const onUseCompiled = useCallback(() => {
    if (compiled) {
      onEdit(compiled);
    }
  }, [compiled, onEdit]);

  // `setHP` turns the light's heading by the altitude and tilts it down by the longitude.
  const reading: string = `${formatDegrees(-manual.sunLongitude)} up at a bearing of ${formatDegrees(manual.sunAltitude)}`;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sun"}
      description={`Sun ${reading}`}
      icon={<WbSunnyIcon />}
      isOn={isOn}
      toggleLabel={"Show the sun in the sky"}
      onToggle={onToggle}
    >
      <LevelWeatherVectorField
        label={"sun_color"}
        isColor
        value={manual.sunColor}
        onChange={(sunColor) => onEdit({ sunColor: toLevelWeatherTriple(sunColor) })}
      />

      <RenderValueSlider
        label={"sun_altitude"}
        value={manual.sunAltitude}
        {...LEVEL_MANUAL_WEATHER_LIMITS.sunAltitude}
        format={formatDegrees}
        onChange={(sunAltitude: number) => onEdit({ sunAltitude })}
      />

      <RenderValueSlider
        label={"sun_longitude"}
        value={manual.sunLongitude}
        {...LEVEL_MANUAL_WEATHER_LIMITS.sunLongitude}
        format={formatDegrees}
        onChange={(sunLongitude: number) => onEdit({ sunLongitude })}
      />

      <Typography className={"block text-text-secondary"} variant={"caption"}>
        {`setHP stands it ${reading}.`}
      </Typography>

      <LevelWeatherVectorField
        label={"ambient_color"}
        isColor
        value={manual.ambientColor}
        onChange={(ambientColor) => onEdit({ ambientColor: toLevelWeatherTriple(ambientColor) })}
      />

      <LevelWeatherVectorField
        label={"hemisphere_color"}
        isColor
        value={manual.hemisphereColor}
        onChange={(hemisphereColor) => onEdit({ hemisphereColor: toLevelWeatherQuad(hemisphereColor) })}
      />

      <Button size={"small"} disabled={!compiled} onClick={onUseCompiled}>
        {compiled ? "Use the level's compiled sun" : "This level names no sun"}
      </Button>

      <LevelWeatherResetButton keys={SUN_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
