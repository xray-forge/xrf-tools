import { ListItemText, MenuItem, TextField } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ChangeEvent, ReactElement } from "react";

import { ILevelWeatherCycleChoice } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { CONTROL } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherCycleSelectProps extends BaseComponentProps {
  cycles: ReadonlyArray<ILevelWeatherCycleChoice>;
  /** The cycle chosen, or null for none. */
  selected: Nullable<string>;
  /** The cycle being read, or null while none is. */
  reading: Nullable<string>;
  onSelect: (name: string) => void;
}

/**
 * Every cycle the level can be played under, the ones its weather resolves to first and marked.
 */
export function LevelWeatherCycleSelect({
  "data-testid": dataTestId = "level-weather-cycle-select",
  id,
  className,
  cycles,
  selected,
  reading,
  onSelect,
}: ILevelWeatherCycleSelectProps): ReactElement {
  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      select
      fullWidth
      size={"small"}
      label={reading ? `Cycle, reading ${reading}` : "Cycle"}
      value={selected ?? ""}
      disabled={reading !== null || !cycles.length}
      slotProps={{
        inputLabel: { shrink: true },
        select: {
          MenuProps: { slotProps: { paper: { sx: { maxHeight: CONTROL.selectMenuMaxHeight } } } },
          renderValue: (value: unknown) => String(value),
        },
      }}
      onChange={(event: ChangeEvent<HTMLInputElement>) => onSelect(event.target.value)}
    >
      {cycles.map((cycle: ILevelWeatherCycleChoice) => (
        <MenuItem key={cycle.name} dense value={cycle.name}>
          <ListItemText
            primary={cycle.isOffered ? `${cycle.name} · level` : cycle.name}
            secondary={[
              `${cycle.keyframes} keyframes`,
              ...cycle.states,
              cycle.findings ? `${cycle.findings} findings` : null,
            ]
              .filter(Boolean)
              .join(" · ")}
          />
        </MenuItem>
      ))}
    </TextField>
  );
}
