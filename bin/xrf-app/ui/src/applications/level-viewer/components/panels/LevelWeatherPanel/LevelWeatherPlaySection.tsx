import { LevelWeatherCycle } from "@xrf/ipc/types/xrf-app-levels";
import { WorldWeatherEffectReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";
import { EditorPanelSection } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material";
import { ReactElement } from "react";

import { LevelWeatherCycleSelect } from "@/core/level/components/weather/LevelWeatherCycleSelect";
import { LevelWeatherEffectSelect } from "@/core/level/components/weather/LevelWeatherEffectSelect";
import { LevelWeatherSeedNote } from "@/core/level/components/weather/LevelWeatherSeedNote";
import { LevelWeatherSourceChoice } from "@/core/level/components/weather/LevelWeatherSourceChoice";
import { ILevelWeatherCycleChoice } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";

interface ILevelWeatherPlaySectionProps extends BaseComponentProps {
  source: ELevelWeatherSource;
  /** Whether a cycle is ready to play. */
  isPlayable: boolean;
  /** Whether the keyframe set by hand lights the level. */
  isManual: boolean;
  /** What the keyframe set by hand was seeded from, or null for none the level played. */
  seed: Nullable<ILevelWeatherSeed>;
  /** Why the last cycle asked for does not play, or null. */
  failure: Nullable<string>;
  /** The level's `weathers` where it leads to no cycle of its game, which its scripts read their own way; or null. */
  unfollowed: Nullable<string>;
  cycles: ReadonlyArray<ILevelWeatherCycleChoice>;
  /** The cycle chosen, or null for none. */
  cycle: Nullable<string>;
  /** The cycle being read, or null while none is. */
  reading: Nullable<string>;
  effects: ReadonlyArray<LevelWeatherCycle>;
  /** The effect playing, or null for none. */
  effect: Nullable<WorldWeatherEffectReport>;
  onSource: (source: ELevelWeatherSource) => void;
  onCycle: (name: string) => void;
  onEffect: (name: Nullable<string>) => void;
}

/**
 * What lights the level: the weather or the keyframe set by hand, the cycle, and the effect over it.
 */
export function LevelWeatherPlaySection({
  "data-testid": dataTestId = "level-weather-play-section",
  id,
  className,
  source,
  isPlayable,
  isManual,
  seed,
  failure,
  unfollowed,
  cycles,
  cycle,
  reading,
  effects,
  effect,
  onSource,
  onCycle,
  onEffect,
}: ILevelWeatherPlaySectionProps): ReactElement {
  return (
    <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Lit by"} isFirst>
      <div className={"flex flex-col gap-3"}>
        <LevelWeatherSourceChoice source={source} isPlayable={isPlayable} onChange={onSource} />

        {failure ? (
          <Typography className={"block wrap-anywhere text-warning"} variant={"caption"}>
            {failure}
          </Typography>
        ) : null}

        {unfollowed ? (
          <Typography className={"block wrap-anywhere text-text-secondary"} variant={"caption"}>
            {`The level's weathers = ${unfollowed} names no cycle here, so the game's scripts choose. Its first cycle plays.`}
          </Typography>
        ) : null}

        {isManual ? (
          <LevelWeatherSeedNote
            seed={seed}
            isPlayable={isPlayable}
            onBack={() => onSource(ELevelWeatherSource.WEATHER)}
          />
        ) : null}

        <LevelWeatherCycleSelect cycles={cycles} selected={cycle} reading={reading} onSelect={onCycle} />

        <LevelWeatherEffectSelect effects={effects} playing={effect} onPlay={onEffect} />
      </div>
    </EditorPanelSection>
  );
}
