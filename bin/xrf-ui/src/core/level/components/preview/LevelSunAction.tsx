import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";
import { Button, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelSunDescription } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherSunSelect } from "@/core/level/components/weather/LevelWeatherSunSelect";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelManualWeather, toLevelManualSun } from "@/core/level/lib/weather/level-manual-weather";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { formatDegrees } from "@/lib/format/angle";

/** The keys the popover sets. */
const SUN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "sunColor",
  "sunAltitude",
  "sunLongitude",
  "ambientColor",
  "hemisphereColor",
  "sun",
  "sunShaftsIntensity",
];

interface ILevelSunActionProps extends Omit<ILevelManualWeatherActionProps, "isOn" | "onToggle"> {
  /** The sun the open level was compiled against, or null where it names none. */
  sun: Nullable<LevelSunDescription>;
  /** The sun or moon the sky draws now, by its `suns.ltx` section, or null where it shows neither. */
  drawnSun: Nullable<string>;
  /** Every lens flare of the game a keyframe can name by `sun`. */
  suns: ReadonlyArray<string>;
  /** The view's switches, of which the lens flares and the sunshafts are turned over here. */
  options: Pick<ILevelViewOptions, "isLensFlared" | "isSunShafted">;
  onToggle: (option: "isLensFlared" | "isSunShafted") => void;
}

/**
 * The sun, the ambient and the hemisphere, by the keys a weather writes them with; the lens flare the sun is drawn
 * with and the light shafts through its shadow, each with its switch. Its direction is marked among the overlays.
 */
export function LevelSunAction({
  "data-testid": dataTestId = "level-sun-action",
  id,
  className,
  manual,
  sun,
  drawnSun,
  suns,
  options,
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
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Sun"}
      description={drawnSun ? `Sun ${reading}, ${drawnSun} in the sky` : `Sun ${reading}, none in the sky`}
      icon={<WbSunnyIcon />}
      isActive={drawnSun !== null}
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

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Lens flare
        </Typography>

        <CheckboxFormRow
          label={"Lens flares"}
          description={"The flares over the frame; the sun's sprite and glow are drawn either way"}
          isChecked={options.isLensFlared}
          onChange={() => onToggle("isLensFlared")}
        />

        <LevelWeatherSunSelect value={manual.sun} suns={suns} onChange={(sun: string) => onEdit({ sun })} />

        <Typography className={"text-text-secondary"} variant={"overline"}>
          Sunshafts
        </Typography>

        <CheckboxFormRow
          label={"Sunshafts"}
          description={"The sun's light through the air, cut by its shadow"}
          isChecked={options.isSunShafted}
          onChange={() => onToggle("isSunShafted")}
        />

        <LevelManualWeatherSlider
          field={"sunShaftsIntensity"}
          manual={manual}
          format={(value: number) => value.toFixed(2)}
          onEdit={onEdit}
        />

        <LevelWeatherResetButton keys={SUN_KEYS} onEdit={onEdit} />
      </div>
    </EditorPopoverAction>
  );
}
