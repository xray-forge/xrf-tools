import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { Button, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelManualWeather, toLevelManualSun } from "@/core/level/lib/weather/level-manual-weather";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { formatDegrees } from "@/lib/format/angle";

/** The keys the popover sets. */
const SUN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "sunColor",
  "sunAltitude",
  "sunLongitude",
  "ambientColor",
  "hemisphereColor",
];

interface ILevelSunActionProps extends Omit<ILevelManualWeatherActionProps, "isOn" | "onToggle"> {
  /** The sun the open level was compiled against, or null where it names none. */
  sun: Nullable<LevelSunDescription>;
}

/**
 * The sun, the ambient and the hemisphere, by the keys a weather writes them with. The weather's sun is drawn with the
 * sky, and its direction marked among the overlays.
 */
export function LevelSunAction({
  "data-testid": dataTestId = "level-sun-action",
  id,
  className,
  manual,
  sun,
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
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sun"}
      description={`Sun ${reading}`}
      icon={<WbSunnyIcon />}
    >
      <div className={"flex w-60 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Sun
        </Typography>

        <LevelManualWeatherVectorField field={"sunColor"} isColor manual={manual} onEdit={onEdit} />

        <LevelManualWeatherSlider field={"sunAltitude"} manual={manual} format={formatDegrees} onEdit={onEdit} />

        <LevelManualWeatherSlider field={"sunLongitude"} manual={manual} format={formatDegrees} onEdit={onEdit} />

        <Typography className={"block text-text-secondary"} variant={"caption"}>
          {`setHP stands it ${reading}.`}
        </Typography>

        <LevelManualWeatherVectorField field={"ambientColor"} isColor manual={manual} onEdit={onEdit} />

        <LevelManualWeatherVectorField field={"hemisphereColor"} isColor manual={manual} onEdit={onEdit} />

        <Button size={"small"} disabled={!compiled} onClick={onUseCompiled}>
          {compiled ? "Use the level's compiled sun" : "This level names no sun"}
        </Button>

        <LevelWeatherResetButton keys={SUN_KEYS} onEdit={onEdit} />
      </div>
    </EditorPopoverAction>
  );
}
