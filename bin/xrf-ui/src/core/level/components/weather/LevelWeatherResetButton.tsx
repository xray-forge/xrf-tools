import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { DEFAULT_LEVEL_MANUAL_WEATHER, ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWeatherResetButtonProps extends BaseComponentProps {
  /** The keys it sets back. */
  keys: ReadonlyArray<keyof ILevelManualWeather>;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * Sets a popover's keys back to `default_clear`'s noon.
 */
export function LevelWeatherResetButton({
  "data-testid": dataTestId = "level-weather-reset-button",
  id,
  className,
  keys,
  onEdit,
}: ILevelWeatherResetButtonProps): ReactElement {
  const onClick = useCallback(
    () => onEdit(Object.fromEntries(keys.map((key) => [key, DEFAULT_LEVEL_MANUAL_WEATHER[key]]))),
    [keys, onEdit]
  );

  return (
    <Button data-testid={dataTestId} id={id} className={className} size={"small"} onClick={onClick}>
      Back to default_clear&apos;s noon
    </Button>
  );
}
