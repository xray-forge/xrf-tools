import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable, runInAction } from "@wirestate/mobx";
import { IRendererWeather, IRendererWeatherReport } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import {
  LevelWeatherCycle,
  LevelWeatherDescription,
  SelectedLevelDescription,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { EWeatherCycleKind } from "@/core/ipc/types/xrf-environment";
import { toLevelRendererWeather } from "@/core/level/lib/weather/level-renderer-weather";
import {
  DEFAULT_LEVEL_WEATHER_CONTROL,
  ILevelWeatherControl,
  LEVEL_WEATHER_NOON,
} from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherEffectRequest } from "@/core/level/lib/weather/level-weather-effect-request";
import {
  ILevelWeatherMemory,
  LEVEL_WEATHER_FACTOR_LIMITS,
  readLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ILevelWeatherSeek } from "@/core/level/lib/weather/level-weather-seek";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { Logger } from "@/lib/logging";

/** Why the toolbar's sun, fog and wind settings do nothing while a weather lights the level. */
const LIGHTING_LOCK: string = "The weather sets it. Switch the Weather panel to Manual to set it by hand";

/**
 * The open level's weather: what it plays, how, and where the renderer's clock stands. The renderer plays the cycle
 * by itself; this only says which cycle and how, hears the time back, and remembers it per level.
 */
@Injectable()
export class LevelWeatherService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The open level's weather, or null while none is read. */
  @RefObservable()
  public description: Nullable<LevelWeatherDescription> = null;

  /** The cycle chosen, or null for none. */
  @RefObservable()
  public cycle: Nullable<LevelWeatherCycle> = null;

  /** The chosen cycle as the renderer plays it, or null while it is still being asked for or there is none. */
  @RefObservable()
  public playable: Nullable<IRendererWeather> = null;

  /** Why the last cycle asked for does not play, or null. */
  @RefObservable()
  public failure: Nullable<string> = null;

  /** The cycle being read, or null while none is. */
  @RefObservable()
  public reading: Nullable<string> = null;

  @RefObservable()
  public source: ELevelWeatherSource = ELevelWeatherSource.WEATHER;

  @RefObservable()
  public control: ILevelWeatherControl = DEFAULT_LEVEL_WEATHER_CONTROL;

  /** The last time the weather was sent to, or null for none since the level opened. */
  @RefObservable()
  public seek: Nullable<ILevelWeatherSeek> = null;

  /** Where the clock stands, in seconds since midnight: sent to, or heard back from the renderer. */
  @RefObservable()
  public time: number = LEVEL_WEATHER_NOON;

  /** The last weather effect asked for, one object a request, or null for none since the level opened. */
  @RefObservable()
  public effect: Nullable<ILevelWeatherEffectRequest> = null;

  /** Where the renderer's weather stood when it last reported, or null before it has. */
  @RefObservable()
  public report: Nullable<IRendererWeatherReport> = null;

  /** The level whose weather is held or being read. */
  private sessionId: Nullable<string> = null;
  /** What the level's weather is remembered under. */
  private memoryKey: Nullable<string> = null;
  /** The level open, which a cycle is read against. */
  private selected: Nullable<SessionSnapshot<SelectedLevelDescription>> = null;

  /** What the renderer plays, or null while the level is lit by hand. */
  public get weather(): Nullable<IRendererWeather> {
    return this.source === ELevelWeatherSource.WEATHER ? this.playable : null;
  }

  /** Why the toolbar's lighting settings do nothing now, or null while they light the level. */
  public get lightingLock(): Nullable<string> {
    return this.weather ? LIGHTING_LOCK : null;
  }

  /**
   * Reads a level's weather and plays the cycle it was last played with, or the first it offers, once per level.
   *
   * @param selected - The level open now, or null for none.
   */
  public async open(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): Promise<void> {
    const sessionId: Nullable<string> = selected?.sessionId ?? null;

    if (sessionId === this.sessionId) {
      return;
    }

    this.clear();

    if (!selected) {
      return;
    }

    const memoryKey: string = toLevelWeatherMemoryKey(selected.value);
    const memory: Nullable<ILevelWeatherMemory> = readLevelWeatherMemory(memoryKey);

    this.sessionId = selected.sessionId;
    this.memoryKey = memoryKey;
    this.selected = selected;

    if (memory) {
      runInAction(() => {
        this.source = memory.source;
        this.control = memory.control;
        this.time = memory.time;
      });
    }

    try {
      const { value: description }: SessionSnapshot<LevelWeatherDescription> = await levelsCommands.readLevelWeather(
        selected.sessionId
      );

      if (this.sessionId !== selected.sessionId) {
        return;
      }

      runInAction(() => {
        this.description = description;
      });

      const first: Maybe<LevelWeatherCycle> = description.offered[0];

      if (!first) {
        throw new Error("The level's weathers resolve to no cycle the game has");
      }

      // A remembered cycle the game no longer has gives way to the level's own.
      if (!memory || !(await this.play(selected, memory.cycle, false))) {
        await this.play(selected, first.name, true);
      }
    } catch (error: unknown) {
      this.fail(selected.sessionId, error);
    }
  }

  /**
   * Plays another cycle from where the clock stands.
   *
   * @param name - The cycle's name.
   */
  public async selectCycle(name: string): Promise<void> {
    const sessionId: Nullable<string> = this.sessionId;
    const selected: Nullable<SessionSnapshot<SelectedLevelDescription>> = this.selected;

    if (!sessionId || !selected || this.cycle?.name === name) {
      return;
    }

    try {
      await this.play(selected, name, true);
      this.persist();
    } catch (error: unknown) {
      this.fail(sessionId, error);
    }
  }

  @BoundAction()
  public setSource(source: ELevelWeatherSource): void {
    this.source = source;
    this.persist();
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
    this.control = {
      ...this.control,
      factor: Math.min(Math.max(Math.round(factor), LEVEL_WEATHER_FACTOR_LIMITS.min), LEVEL_WEATHER_FACTOR_LIMITS.max),
    };
    this.persist();
  }

  @BoundAction()
  public setDynamicSun(isDynamicSun: boolean): void {
    this.control = { ...this.control, isDynamicSun };
    this.persist();
  }

  /**
   * @param name - The effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  @BoundAction()
  public playEffect(name: Nullable<string>): void {
    this.effect = { name };
  }

  /**
   * @param time - Seconds since midnight to play on from.
   */
  @BoundAction()
  public seekTo(time: number): void {
    this.seek = { time };
    this.time = time;
    this.persist();
  }

  /**
   * @param report - Where the renderer's weather stood when it reported, or null while none plays.
   */
  @BoundAction()
  public noteReport(report: Nullable<IRendererWeatherReport>): void {
    this.report = report;

    if (report && report.time !== this.time) {
      this.time = report.time;
    }
  }

  /** Remembers how the level's weather played, then forgets it all, for the next level to start from its own. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.persist();
    this.sessionId = null;
    this.memoryKey = null;
    this.selected = null;
    this.description = null;
    this.cycle = null;
    this.playable = null;
    this.failure = null;
    this.reading = null;
    this.seek = null;
    this.effect = null;
    this.report = null;
    this.source = ELevelWeatherSource.WEATHER;
    this.control = DEFAULT_LEVEL_WEATHER_CONTROL;
    this.time = LEVEL_WEATHER_NOON;
  }

  /**
   * Reads a cycle and asks where its skies are fetched from, then plays it, for a level still open.
   *
   * @param selected - The level it plays in.
   * @param name - The cycle's name.
   * @param isRequired - Whether a cycle that cannot be read is a failure, rather than an answer of false.
   * @returns Whether it plays.
   */
  private async play(
    selected: SessionSnapshot<SelectedLevelDescription>,
    name: string,
    isRequired: boolean
  ): Promise<boolean> {
    runInAction(() => {
      this.reading = name;
    });

    try {
      const cycle: LevelWeatherCycle =
        this.description?.offered.find((it: LevelWeatherCycle) => it.name === name) ??
        (await levelsCommands.readLevelCycle(selected.sessionId, { kind: EWeatherCycleKind.CYCLE, name })).value;
      const description: Nullable<LevelWeatherDescription> = this.description;

      if (!description) {
        return false;
      }

      const playable: IRendererWeather = await toLevelRendererWeather({
        cycle,
        description,
        roots: selected.value.roots,
      });

      if (this.sessionId !== selected.sessionId || this.reading !== name) {
        return true;
      }

      runInAction(() => {
        this.cycle = cycle;
        this.playable = playable;
        this.failure = null;
        this.reading = null;
      });

      return true;
    } catch (error: unknown) {
      if (isRequired) {
        throw error;
      }

      this.log.warn(`The remembered cycle '${name}' is not played:`, transformError(error).message);

      return false;
    }
  }

  private fail(sessionId: string, error: unknown): void {
    const reason: string = transformError(error).message;

    this.log.warn("The level's weather is not played:", reason);

    if (this.sessionId === sessionId) {
      runInAction(() => {
        this.failure = reason;
        this.reading = null;
      });
    }
  }

  /** Remembers how the open level's weather plays, once a cycle does. */
  private persist(): void {
    if (this.memoryKey && this.cycle) {
      writeLevelWeatherMemory(this.memoryKey, {
        control: this.control,
        cycle: this.cycle.name,
        source: this.source,
        time: this.time,
      });
    }
  }
}
