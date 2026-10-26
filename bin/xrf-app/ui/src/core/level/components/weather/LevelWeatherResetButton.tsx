import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      Reset to the weather
    </Button>
  );
}
