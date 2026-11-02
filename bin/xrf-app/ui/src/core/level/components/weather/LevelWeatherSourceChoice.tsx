import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ToggleButton, ToggleButtonGroup } from "@xrf/ui/material/toggle-button";
import { ReactElement } from "react";

import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";

interface ILevelWeatherSourceChoiceProps extends BaseComponentProps {
  source: ELevelWeatherSource;
  /** Whether a cycle is ready to play. */
  isPlayable: boolean;
  onChange: (source: ELevelWeatherSource) => void;
}

/**
 * Whether the level's weather lights it, or the keyframe set by hand; the keyframe while no cycle plays.
 */
export function LevelWeatherSourceChoice({
  "data-testid": dataTestId = "level-weather-source-choice",
  id,
  className,
  source,
  isPlayable,
  onChange,
}: ILevelWeatherSourceChoiceProps): ReactElement {
  return (
    <ToggleButtonGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      exclusive
      fullWidth
      color={"primary"}
      size={"small"}
      value={isPlayable ? source : ELevelWeatherSource.MANUAL}
      aria-label={"Lit by"}
      onChange={(_, next: Nullable<ELevelWeatherSource>) => {
        if (next !== null) {
          onChange(next);
        }
      }}
    >
      <ToggleButton value={ELevelWeatherSource.WEATHER} disabled={!isPlayable}>
        Weather
      </ToggleButton>
      <ToggleButton value={ELevelWeatherSource.MANUAL}>Manual</ToggleButton>
    </ToggleButtonGroup>
  );
}
