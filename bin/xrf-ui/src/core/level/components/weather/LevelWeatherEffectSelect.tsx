import { ListItemText, MenuItem, TextField } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ChangeEvent, ReactElement } from "react";

import { LevelWeatherCycle } from "@/core/ipc/types/xrf-app";
import { WorldWeatherEffectReport } from "@/core/ipc/types/xrf-world";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { CONTROL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What the select holds while no effect plays. */
const NONE: string = "";

interface ILevelWeatherEffectSelectProps extends BaseComponentProps {
  effects: ReadonlyArray<LevelWeatherCycle>;
  /** The effect playing, or null for none. */
  playing: Nullable<WorldWeatherEffectReport>;
  isDisabled?: boolean;
  onPlay: (name: Nullable<string>) => void;
}

/**
 * The game's weather effects, one of which plays over what lights the level until it gives it back; None stops it.
 */
export function LevelWeatherEffectSelect({
  "data-testid": dataTestId = "level-weather-effect-select",
  id,
  className,
  effects,
  playing,
  isDisabled = false,
  onPlay,
}: ILevelWeatherEffectSelectProps): ReactElement {
  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      select
      fullWidth
      size={"small"}
      label={"Effect"}
      value={playing?.name ?? NONE}
      disabled={isDisabled || !effects.length}
      helperText={playing ? `${formatLevelWeatherTime(playing.remaining ?? 0, true)} of game time left` : undefined}
      slotProps={{
        inputLabel: { shrink: true },
        select: {
          displayEmpty: true,
          MenuProps: { slotProps: { paper: { sx: { maxHeight: CONTROL.selectMenuMaxHeight } } } },
          renderValue: (value: unknown) => String(value) || "None",
        },
      }}
      onChange={(event: ChangeEvent<HTMLInputElement>) => onPlay(event.target.value || null)}
    >
      <MenuItem dense value={NONE}>
        None
      </MenuItem>
      {effects.map((effect: LevelWeatherCycle) => (
        <MenuItem key={effect.name} dense value={effect.name}>
          <ListItemText
            primary={effect.name}
            secondary={[
              `${effect.keyframes.length} keyframes`,
              effect.findings.length ? `${effect.findings.length} findings` : null,
            ]
              .filter(Boolean)
              .join(" · ")}
          />
        </MenuItem>
      ))}
    </TextField>
  );
}
