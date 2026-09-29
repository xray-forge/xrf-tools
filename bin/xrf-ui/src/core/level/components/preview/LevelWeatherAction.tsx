import { default as FilterDramaIcon } from "@mui/icons-material/FilterDrama";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

import { LevelWeatherClock } from "@/core/level/components/weather/LevelWeatherClock";
import { LevelWeatherCycleSelect } from "@/core/level/components/weather/LevelWeatherCycleSelect";
import { LevelWeatherEffectSelect } from "@/core/level/components/weather/LevelWeatherEffectSelect";
import { LevelWeatherSeedNote } from "@/core/level/components/weather/LevelWeatherSeedNote";
import { LevelWeatherSourceChoice } from "@/core/level/components/weather/LevelWeatherSourceChoice";
import { toLevelManualWeatherLtx } from "@/core/level/lib/weather/level-manual-weather-ltx";
import { listLevelWeatherCycles } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Logger, useLogger } from "@/lib/logging";

/** Milliseconds the copy button says it copied. */
const COPIED_FOR: number = 1500;

/**
 * The weather in short: what lights the level, the cycle and the effect, and the clock; the panel keeps the detail.
 */
export function LevelWeatherAction({
  "data-testid": dataTestId = "level-weather-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const [isCopied, setIsCopied] = useState<boolean>(false);

  const { description, cycle, source, time, control, report, isManual, seed } = weatherService;
  const isPlayable: boolean = weatherService.playable !== null;
  const isPlaying: boolean = !control.isPaused;
  const cycles = useMemo(() => (description ? listLevelWeatherCycles(description) : []), [description]);

  const onCopy = useCallback(() => {
    const text: string = toLevelManualWeatherLtx({
      engine: weatherService.engine,
      manual: weatherService.shown,
      time: weatherService.time,
    });

    navigator.clipboard
      ?.writeText(text)
      .then(() => setIsCopied(true))
      .catch((error: unknown) => log.error("Failed to copy the keyframe:", error));
  }, [log, weatherService]);

  useEffect(() => {
    if (!isCopied) {
      return;
    }

    const timeout: ReturnType<typeof setTimeout> = setTimeout(() => setIsCopied(false), COPIED_FOR);

    return () => clearTimeout(timeout);
  }, [isCopied]);

  const at: string = formatLevelWeatherTime(time);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Weather"}
      description={
        isManual
          ? `Lit by hand at ${at}${isPlaying ? ", running" : ""}`
          : `${cycle?.name ?? "Weather"} at ${at}${isPlaying ? ", running" : ""}`
      }
      icon={<FilterDramaIcon />}
      isActive={!isManual}
    >
      <div className={"flex w-72 flex-col gap-3 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Weather
        </Typography>

        <LevelWeatherSourceChoice source={source} isPlayable={isPlayable} onChange={weatherService.setSource} />

        {isManual ? (
          <LevelWeatherSeedNote
            seed={seed}
            isPlayable={isPlayable}
            onBack={() => weatherService.setSource(ELevelWeatherSource.WEATHER)}
          />
        ) : null}

        <LevelWeatherCycleSelect
          cycles={cycles}
          selected={cycle?.name ?? null}
          reading={weatherService.reading}
          onSelect={(name: string) => void weatherService.selectCycle(name)}
        />

        <LevelWeatherEffectSelect
          effects={description?.effects ?? []}
          playing={report?.effect ?? null}
          onPlay={weatherService.playEffect}
        />

        <LevelWeatherClock
          time={time}
          isPlaying={isPlaying}
          onSeek={weatherService.seekTo}
          onPlaying={weatherService.setPlaying}
        />

        <Button size={"small"} onClick={onCopy}>
          {isCopied ? "Copied" : "Copy as LTX"}
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
