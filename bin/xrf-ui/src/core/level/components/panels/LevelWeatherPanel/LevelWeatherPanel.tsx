import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useMemo } from "react";

import { LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { LevelWeatherClockSection } from "@/core/level/components/panels/LevelWeatherPanel/LevelWeatherClockSection";
import { LevelWeatherFindingsSection } from "@/core/level/components/panels/LevelWeatherPanel/LevelWeatherFindingsSection";
import { LevelWeatherModifiersSection } from "@/core/level/components/panels/LevelWeatherPanel/LevelWeatherModifiersSection";
import { LevelWeatherPlaySection } from "@/core/level/components/panels/LevelWeatherPanel/LevelWeatherPlaySection";
import { ILevelWeatherCycleChoice, listLevelWeatherCycles } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { LevelLoadService, LevelWeatherService } from "@/core/level/services";
import { EditorPanel, EditorPanelEmpty } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * The level's weather: what lights it, the time of day and how it runs, the sun, the effects over it, the level's own
 * overrides, the cycle, and what is wrong in it.
 */
export function LevelWeatherPanel({
  "data-testid": dataTestId = "level-weather-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);

  const description: Nullable<LevelWeatherDescription> = weatherService.description;
  const cycle: Nullable<LevelWeatherCycle> = weatherService.cycle;
  const isWeather: boolean = weatherService.weather !== null;
  const isManual: boolean = weatherService.isManual;

  const cycles: Array<ILevelWeatherCycleChoice> = useMemo(
    () => (description ? listLevelWeatherCycles(description) : []),
    [description]
  );
  const keyframes: Array<number> = useMemo(() => cycle?.keyframes.map((it) => it.time) ?? [], [cycle]);

  const onSelectCycle = useCallback((name: string) => void weatherService.selectCycle(name), [weatherService]);

  if (!loadService.level.value) {
    return (
      <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Weather"}>
        <EditorPanelEmpty label={"No level open. Open one to play its weather."} />
      </EditorPanel>
    );
  }

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Weather"}>
      <LevelWeatherPlaySection
        source={weatherService.source}
        isPlayable={weatherService.playable !== null}
        isManual={isManual}
        seed={weatherService.seed}
        failure={weatherService.failure}
        cycles={cycles}
        cycle={cycle?.name ?? null}
        reading={weatherService.reading}
        effects={description?.effects ?? []}
        effect={isWeather ? (weatherService.report?.effect ?? null) : null}
        onSource={weatherService.setSource}
        onCycle={onSelectCycle}
        onEffect={weatherService.playEffect}
      />

      {description ? (
        <>
          <LevelWeatherClockSection
            time={weatherService.time}
            keyframes={keyframes}
            report={isWeather ? weatherService.report : null}
            control={weatherService.control}
            engine={description.engine}
            isManual={isManual}
            onSeek={weatherService.seekTo}
            onPlaying={weatherService.setPlaying}
            onFactor={weatherService.setFactor}
            onDynamicSun={weatherService.setDynamicSun}
          />

          <LevelWeatherModifiersSection
            count={description.modifiers.length}
            reaching={isWeather ? (weatherService.report?.modifiers ?? 0) : 0}
          />

          {cycle ? <LevelWeatherFindingsSection file={cycle.file} findings={cycle.findings} /> : null}
        </>
      ) : (
        <EditorPanelEmpty
          label={weatherService.failure ? "The level's weather is not read." : "Reading the weather."}
        />
      )}
    </EditorPanel>
  );
}
