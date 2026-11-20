import { BoundAction, comparer, makeObservable, RefObservable } from "@wirestate/mobx";
import { WorldWeatherReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import {
  DEFAULT_LEVEL_WEATHER_CONTROL,
  ILevelWeatherControl,
  LEVEL_WEATHER_NOON,
  toLevelWeatherFactor,
} from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherMemory } from "@/core/level/lib/weather/level-weather-memory";

/**
 * The level's weather clock: how it runs, where it stands, and what the renderer last reported of it. The renderer owns
 * the clock; this is what it was asked to do and what it said back.
 */
export class LevelWeatherClock {
  @RefObservable()
  public control: ILevelWeatherControl = DEFAULT_LEVEL_WEATHER_CONTROL;

  /** Where the clock stands, in seconds since midnight: sent to, or heard back from the renderer. */
  @RefObservable()
  public time: number = LEVEL_WEATHER_NOON;

  /** Where the renderer's weather stood when it last reported, or null before it has. */
  @RefObservable()
  public report: Nullable<WorldWeatherReport> = null;

  /**
   * @param persist - Remembers the level's weather, after each change to the clock asked for by hand.
   */
  public constructor(private readonly persist: () => void) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  @BoundAction()
  public setPlaying(isPlaying: boolean): void {
    this.control = { ...this.control, isPaused: !isPlaying };
    this.persist();
  }

  /**
   * @param factor - Game seconds a real second, kept within the engine's bounds.
   */
  @BoundAction()
  public setFactor(factor: number): void {
    this.control = { ...this.control, factor: toLevelWeatherFactor(factor) };
    this.persist();
  }

  @BoundAction()
  public setDynamicSun(isDynamicSun: boolean): void {
    this.control = { ...this.control, isDynamicSun };
    this.persist();
  }

  /**
   * Notes where the clock was sent to, which is where it stands until the renderer reports otherwise.
   *
   * @param time - Seconds since midnight to play on from.
   */
  @BoundAction()
  public seekTo(time: number): void {
    this.time = time;
    this.persist();
  }

  /**
   * @param report - Where the renderer's weather stood when it reported, or null while none plays; one the same as the
   *   last is dropped, so a paused clock redraws nothing that reads it.
   */
  @BoundAction()
  public noteReport(report: Nullable<WorldWeatherReport>): void {
    if (!comparer.structural(report, this.report)) {
      this.report = report;
    }

    const time: Nullable<number> = report?.time ?? null;

    if (time !== null && time !== this.time) {
      this.time = time;
    }
  }

  /** Stops the clock for something borrowing the weather, without remembering it. */
  @BoundAction()
  public hold(): void {
    this.control = { ...this.control, isPaused: true };
  }

  /**
   * Runs the clock as it ran before it was held; where it stood is the caller's to seek, as the renderer owns it.
   *
   * @param control - How it ran when it was held.
   */
  @BoundAction()
  public release(control: ILevelWeatherControl): void {
    this.control = control;
  }

  /**
   * @param memory - How the level's clock ran and where it stood when its weather was last played.
   */
  @BoundAction()
  public restore(memory: Pick<ILevelWeatherMemory, "control" | "time">): void {
    this.control = memory.control;
    this.time = memory.time;
  }

  /** Forgets the level's clock, for the next level to start from its own. */
  @BoundAction()
  public reset(): void {
    this.report = null;
    this.control = DEFAULT_LEVEL_WEATHER_CONTROL;
    this.time = LEVEL_WEATHER_NOON;
  }
}
