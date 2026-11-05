import { Nullable } from "@xrf/types";
import { EditorPopoverNote } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";

interface ILevelWeatherSeedNoteProps extends BaseComponentProps {
  /** What the keyframe set by hand was seeded from, or null for none the level played. */
  seed: Nullable<ILevelWeatherSeed>;
}

/**
 * What the keyframe set by hand lighting the level was seeded from; the source's Weather choice above it goes back.
 */
export function LevelWeatherSeedNote({
  "data-testid": dataTestId = "level-weather-seed-note",
  id,
  className,
  seed,
}: ILevelWeatherSeedNoteProps): ReactElement {
  return (
    <EditorPopoverNote data-testid={dataTestId} id={id} className={className}>
      {seed
        ? `Seeded from ${seed.cycle} at ${formatLevelWeatherTime(seed.time)}`
        : `Seeded from default_clear at ${formatLevelWeatherTime(LEVEL_WEATHER_NOON)}`}
    </EditorPopoverNote>
  );
}
