import { BaseComponentProps } from "@xrf/ui/element-types";
import { ListItemText } from "@xrf/ui/material/list";
import { MenuItem } from "@xrf/ui/material/menu";
import { TextField } from "@xrf/ui/material/text-field";
import { CONTROL } from "@xrf/ui/theme";
import { ChangeEvent, ReactElement } from "react";

import {
  describeLevelManualWeatherKey,
  LEVEL_MANUAL_WEATHER_LABELS,
} from "@/core/level/lib/weather/level-manual-weather-labels";

/** What the select holds for a keyframe that draws no sun. */
const NONE: string = "";

interface ILevelWeatherSunSelectProps extends BaseComponentProps {
  /** The lens flare drawn, empty for none. */
  value: string;
  /** Every lens flare of the game, by its section. */
  suns: ReadonlyArray<string>;
  onChange: (sun: string) => void;
}

/**
 * `sun`: the lens flare a keyframe draws, its sprite in the sky, its flares and gradient; None draws none.
 */
export function LevelWeatherSunSelect({
  "data-testid": dataTestId = "level-weather-sun-select",
  id,
  className,
  value,
  suns,
  onChange,
}: ILevelWeatherSunSelectProps): ReactElement {
  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      select
      fullWidth
      size={"small"}
      label={LEVEL_MANUAL_WEATHER_LABELS.sun}
      title={describeLevelManualWeatherKey("sun")}
      value={value}
      disabled={!suns.length && !value}
      slotProps={{
        inputLabel: { shrink: true },
        select: {
          displayEmpty: true,
          MenuProps: { slotProps: { paper: { sx: { maxHeight: CONTROL.selectMenuMaxHeight } } } },
          renderValue: (selected: unknown) => String(selected) || "None",
        },
      }}
      onChange={(event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value)}
    >
      <MenuItem dense value={NONE}>
        None
      </MenuItem>
      {/* A lens flare the game no longer has, kept so the keyframe still says what it names. */}
      {value && !suns.includes(value) ? (
        <MenuItem dense value={value}>
          <ListItemText primary={value} secondary={"not in the game"} />
        </MenuItem>
      ) : null}
      {suns.map((sun: string) => (
        <MenuItem key={sun} dense value={sun}>
          {sun}
        </MenuItem>
      ))}
    </TextField>
  );
}
