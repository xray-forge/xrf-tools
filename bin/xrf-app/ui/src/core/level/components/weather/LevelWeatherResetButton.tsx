import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { ReactElement, useCallback } from "react";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";

/** What a reset of keys set by hand says it does: they stand as the weather they were seeded from again. */
export const LEVEL_WEATHER_RESET_LABEL: string = "Reset to the weather";

interface ILevelWeatherResetButtonProps extends BaseComponentProps {
  /** The keys it sets back. */
  keys: ReadonlyArray<keyof ILevelManualWeather>;
  onReset: (keys: ReadonlyArray<keyof ILevelManualWeather>) => void;
}

/**
 * Sets a popover's keys back to the weather the keyframe set by hand was seeded from.
 */
export function LevelWeatherResetButton({
  "data-testid": dataTestId = "level-weather-reset-button",
  id,
  className,
  keys,
  onReset,
}: ILevelWeatherResetButtonProps): ReactElement {
  const onClick = useCallback(() => onReset(keys), [keys, onReset]);

  return (
    <Button data-testid={dataTestId} id={id} className={className} size={"small"} onClick={onClick}>
      {LEVEL_WEATHER_RESET_LABEL}
    </Button>
  );
}
