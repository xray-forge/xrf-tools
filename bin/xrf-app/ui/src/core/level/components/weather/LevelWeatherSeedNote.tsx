import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";

interface ILevelWeatherSeedNoteProps extends BaseComponentProps {
  /** What the keyframe set by hand was seeded from, or null for none the level played. */
  seed: Nullable<ILevelWeatherSeed>;
  /** Whether a cycle is ready to go back to. */
  isPlayable: boolean;
  onBack: () => void;
}

/**
 * What the keyframe set by hand lighting the level was seeded from, and the way back to the weather.
 */
export function LevelWeatherSeedNote({
  "data-testid": dataTestId = "level-weather-seed-note",
  id,
  className,
  seed,
  isPlayable,
  onBack,
}: ILevelWeatherSeedNoteProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex items-center justify-between gap-2", className)}>
      <Typography className={"min-w-0 wrap-anywhere text-text-secondary"} variant={"caption"}>
        {seed
          ? `Seeded from ${seed.cycle} at ${formatLevelWeatherTime(seed.time)}`
          : `Seeded from default_clear at ${formatLevelWeatherTime(LEVEL_WEATHER_NOON)}`}
      </Typography>

      {isPlayable ? (
        <Button size={"small"} onClick={onBack}>
          Play the weather
        </Button>
      ) : null}
    </div>
  );
}
