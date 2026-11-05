import { useInjection } from "@wirestate/react";
import { LevelWeatherCycle, LevelWeatherDescription } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";
import { useLogger } from "@xrf/logging/react";
import { Nullable } from "@xrf/types";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FilterDramaIcon } from "@xrf/ui/icons/filter-drama";
import { Button } from "@xrf/ui/material/button";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelWeatherCycleSelect } from "@/core/level/components/weather/LevelWeatherCycleSelect";
import { LevelWeatherEffectSelect } from "@/core/level/components/weather/LevelWeatherEffectSelect";
import { LevelWeatherSeedNote } from "@/core/level/components/weather/LevelWeatherSeedNote";
import { LevelWeatherSourceChoice } from "@/core/level/components/weather/LevelWeatherSourceChoice";
import { LevelWeatherTime } from "@/core/level/components/weather/LevelWeatherTime";
import { toLevelManualWeatherLtx } from "@/core/level/lib/weather/level-manual-weather-ltx";
import { listLevelWeatherCycles } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { useCopiedText } from "@/lib/react/use-copied-text";

/**
 * The weather in short: what lights the level, the cycle and the effect, and the time of day as the panel shows it (the
 * clock with the cycle's keyframes marked, the keyframes it stands between, its speed); the panel keeps the detail.
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
  const keyframes: Array<number> = useMemo(() => cycle?.keyframes.map((it) => it.time) ?? [], [cycle]);

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
      <EditorPopoverContent label={"Weather"}>
        <EditorPopoverGroupSection label={"Lit by"}>
          <LevelWeatherSourceChoice source={source} isPlayable={isPlayable} onChange={weatherService.setSource} />

          {isManual ? (
            <LevelWeatherSeedNote
              className={"mt-1"}
              seed={seed}
              isPlayable={isPlayable}
              onBack={() => weatherService.setSource(ELevelWeatherSource.WEATHER)}
            />
          ) : null}
        </EditorPopoverGroupSection>

        <EditorPopoverGroupSection label={"Cycle"}>
          <LevelWeatherCycleSelect
            className={"mt-2"}
            cycles={cycles}
            selected={cycle?.name ?? null}
            reading={weatherService.reading}
            onSelect={(name: string) => void weatherService.selectCycle(name)}
          />

          <LevelWeatherEffectSelect
            className={"mt-3"}
            effects={description?.effects ?? []}
            playing={report?.effect ?? null}
            onPlay={renderService.weather.playEffect}
          />
        </EditorPopoverGroupSection>

        <EditorPopoverGroupSection label={"Time"}>
          <LevelWeatherTime
            time={time}
            keyframes={keyframes}
            report={weatherService.weather !== null ? report : null}
            control={control}
            onSeek={renderService.weather.seek}
            onPlaying={weatherService.setPlaying}
            onFactor={weatherService.setFactor}
          />

          <Button className={"mt-1 self-start"} size={"small"} onClick={onCopy}>
            {isCopied ? "Copied" : "Copy as LTX"}
          </Button>
        </EditorPopoverGroupSection>
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
