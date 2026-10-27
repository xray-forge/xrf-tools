import { useInjection } from "@wirestate/react";
import { LevelWeatherCycle, LevelWeatherDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FilterDramaIcon } from "@xrf/ui/icons";
import { Button, Typography } from "@xrf/ui/material";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelWeatherClock } from "@/core/level/components/weather/LevelWeatherClock";
import { LevelWeatherCycleSelect } from "@/core/level/components/weather/LevelWeatherCycleSelect";
import { LevelWeatherEffectSelect } from "@/core/level/components/weather/LevelWeatherEffectSelect";
import { LevelWeatherSeedNote } from "@/core/level/components/weather/LevelWeatherSeedNote";
import { LevelWeatherSourceChoice } from "@/core/level/components/weather/LevelWeatherSourceChoice";
import { toLevelManualWeatherLtx } from "@/core/level/lib/weather/level-manual-weather-ltx";
import { listLevelWeatherCycles } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { useCopiedText } from "@/lib/react/use-copied-text";

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
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  const { source, time, control, report, isManual, seed } = weatherService;
  const description: Nullable<LevelWeatherDescription> = weatherService.description.value;
  const cycle: Nullable<LevelWeatherCycle> = weatherService.cycle.value;
  const isPlayable: boolean = cycle !== null;
  const isPlaying: boolean = !control.isPaused;
  const cycles = useMemo(() => (description ? listLevelWeatherCycles(description) : []), [description]);

  const at: string = formatLevelWeatherTime(time);

  const onFailure = useCallback((error: unknown) => log.error("Failed to copy the keyframe:", error), [log]);

  const { isCopied, copy } = useCopiedText(onFailure);

  const onCopy = useCallback(
    () =>
      copy(
        toLevelManualWeatherLtx({
          engine: weatherService.engine,
          manual: weatherService.shown,
          time: weatherService.time,
        })
      ),
    [copy, weatherService]
  );

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
          onPlay={renderService.playWeatherEffect}
        />

        <LevelWeatherClock
          time={time}
          isPlaying={isPlaying}
          onSeek={renderService.seekWeather}
          onPlaying={weatherService.setPlaying}
        />

        <Button size={"small"} onClick={onCopy}>
          {isCopied ? "Copied" : "Copy as LTX"}
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
