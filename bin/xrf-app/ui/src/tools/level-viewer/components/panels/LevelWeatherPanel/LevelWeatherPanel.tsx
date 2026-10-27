import { useInjection } from "@wirestate/react";
import { LevelWeatherCycle, LevelWeatherDescription } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngine } from "@xrf/ipc/types/xrf-engine-target";
import { Nullable } from "@xrf/types";
import { EditorPanelEmpty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useMemo } from "react";

import { ILevelWeatherCycleChoice, listLevelWeatherCycles } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { EditorPanel } from "@/core/shell/editor/EditorPanel";
import { LevelWeatherAmbientSection } from "@/tools/level-viewer/components/panels/LevelWeatherPanel/LevelWeatherAmbientSection";
import { LevelWeatherClockSection } from "@/tools/level-viewer/components/panels/LevelWeatherPanel/LevelWeatherClockSection";
import { LevelWeatherFindingsSection } from "@/tools/level-viewer/components/panels/LevelWeatherPanel/LevelWeatherFindingsSection";
import { LevelWeatherModifiersSection } from "@/tools/level-viewer/components/panels/LevelWeatherPanel/LevelWeatherModifiersSection";
import { LevelWeatherPlaySection } from "@/tools/level-viewer/components/panels/LevelWeatherPanel/LevelWeatherPlaySection";

/**
 * The level's weather: what lights it, the time of day and how it runs, the sun, the effects over it, the level's own
 * overrides, the ambient effects near the camera, the cycle, and what is wrong in it.
 */
export function LevelWeatherPanel({
  "data-testid": dataTestId = "level-weather-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  const description: Nullable<LevelWeatherDescription> = weatherService.description.value;
  const cycle: Nullable<LevelWeatherCycle> = weatherService.cycle.value;
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
        isPlayable={cycle !== null}
        isManual={isManual}
        seed={weatherService.seed}
        failure={weatherService.failure}
        unfollowed={weatherService.unfollowed}
        cycles={cycles}
        cycle={cycle?.name ?? null}
        reading={weatherService.reading}
        effects={description?.effects ?? []}
        effect={isWeather ? (weatherService.report?.effect ?? null) : null}
        onSource={weatherService.setSource}
        onCycle={onSelectCycle}
        onEffect={renderService.playWeatherEffect}
      />

      {description ? (
        <>
          <LevelWeatherClockSection
            time={weatherService.time}
            keyframes={keyframes}
            report={isWeather ? weatherService.report : null}
            control={weatherService.control}
            isSunTabled={description.engine === EXrayEngine.EXTENDED && description.sunTable !== null}
            isManual={isManual}
            onSeek={renderService.seekWeather}
            onPlaying={weatherService.setPlaying}
            onFactor={weatherService.setFactor}
            onDynamicSun={weatherService.setDynamicSun}
          />

          <LevelWeatherModifiersSection
            count={description.modifiers.length}
            reaching={isWeather ? (weatherService.report?.modifiers ?? 0) : 0}
          />

          <LevelWeatherAmbientSection
            ambient={isWeather ? (weatherService.report?.ambient ?? null) : null}
            isPlayed={viewService.options.isAmbientPlayed}
            onPlay={renderService.playAmbientEffect}
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
