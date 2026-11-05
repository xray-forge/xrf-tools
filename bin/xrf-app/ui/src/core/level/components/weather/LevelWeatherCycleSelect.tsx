import { Nullable } from "@xrf/types";
import { EditorPopoverSelectRow } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ListItemText } from "@xrf/ui/material/list";
import { MenuItem } from "@xrf/ui/material/menu";
import { ReactElement } from "react";

import {
  formatLevelWeatherFindings,
  formatLevelWeatherKeyframes,
  ILevelWeatherCycleChoice,
} from "@/core/level/lib/weather/level-weather-cycle-choice";

interface ILevelWeatherCycleSelectProps extends BaseComponentProps {
  cycles: ReadonlyArray<ILevelWeatherCycleChoice>;
  /** The cycle chosen, or null for none. */
  selected: Nullable<string>;
  /** The cycle being read, or null while none is. */
  reading: Nullable<string>;
  onSelect: (name: string) => void;
}

/**
 * Every cycle the level can be played under, the ones its weather resolves to first and marked; the one being read
 * named in its place until it plays.
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
    <EditorPopoverSelectRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Cycle"}
      value={selected ?? ""}
      isDisabled={reading !== null || !cycles.length}
      renderValue={(value: string) => (reading ? `Reading ${reading}…` : value)}
      onChange={onSelect}
    >
      {cycles.map((cycle: ILevelWeatherCycleChoice) => (
        <MenuItem key={cycle.name} dense value={cycle.name}>
          <ListItemText
            primary={cycle.isOffered ? `${cycle.name} · level` : cycle.name}
            secondary={[
              formatLevelWeatherKeyframes(cycle.keyframes),
              ...cycle.states,
              formatLevelWeatherFindings(cycle.findings),
            ]
              .filter(Boolean)
              .join(" · ")}
          />
        </MenuItem>
      ))}
    </EditorPopoverSelectRow>
  );
}
