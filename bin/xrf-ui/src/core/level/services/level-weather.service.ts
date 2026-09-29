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
import { toLevelRendererWeather } from "@/core/level/lib/weather/level-renderer-weather";
import {
  DEFAULT_LEVEL_WEATHER_CONTROL,
  ILevelWeatherControl,
  LEVEL_WEATHER_NOON,
} from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherSeek } from "@/core/level/lib/weather/level-weather-seek";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { Logger } from "@/lib/logging";

/**
 * The open level's weather: what it plays, how, and where the renderer's clock stands. The renderer plays the cycle
 * by itself; this only says which cycle, and hears the time back.
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

  /** Why the level is lit by hand though the weather was asked for, or null. */
  @RefObservable()
  public failure: Nullable<string> = null;

  @RefObservable()
  public source: ELevelWeatherSource = ELevelWeatherSource.WEATHER;

  @RefObservable()
  public control: ILevelWeatherControl = DEFAULT_LEVEL_WEATHER_CONTROL;

  /** The last time the weather was sent to, or null for none since the level opened. */
  @RefObservable()
  public seek: Nullable<ILevelWeatherSeek> = null;

  /** Where the renderer's clock stood when it last reported, in seconds since midnight. */
  @RefObservable()
  public time: number = LEVEL_WEATHER_NOON;

  /** The level whose weather is held or being read. */
  private sessionId: Nullable<string> = null;

  /** What the renderer plays, or null while the level is lit by hand. */
  public get weather(): Nullable<IRendererWeather> {
    return this.source === ELevelWeatherSource.WEATHER ? this.playable : null;
  }

  /**
   * Reads a level's weather and plays the first cycle it offers, once per level.
   *
   * @param selected - The level open now, or null for none.
   */
  public async open(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): Promise<void> {
    const sessionId: Nullable<string> = selected?.sessionId ?? null;

    if (sessionId === this.sessionId) {
      return;
    }

    this.clear();
    this.sessionId = sessionId;

    if (!selected) {
      return;
    }

    try {
      const { value: description }: SessionSnapshot<LevelWeatherDescription> = await levelsCommands.readLevelWeather(
        selected.sessionId
      );
      const cycle: Maybe<LevelWeatherCycle> = description.offered[0];

      if (!cycle) {
        throw new Error("The level's weathers resolve to no cycle the game has");
      }

      const playable: IRendererWeather = await toLevelRendererWeather({
        cycle,
        description,
        roots: selected.value.roots,
      });

      if (this.sessionId === sessionId) {
        runInAction(() => {
          this.description = description;
          this.cycle = cycle;
          this.playable = playable;
        });
      }
    } catch (error: unknown) {
      const reason: string = transformError(error).message;

      this.log.warn("The level's weather is not played:", reason);

      if (this.sessionId === sessionId) {
        runInAction(() => {
          this.failure = reason;
        });
      }
    }
  }

  @BoundAction()
  public setSource(source: ELevelWeatherSource): void {
    this.source = source;
  }

  @BoundAction()
  public setControl(control: ILevelWeatherControl): void {
    this.control = control;
  }

  /**
   * @param time - Seconds since midnight to play on from.
   */
  @BoundAction()
  public seekTo(time: number): void {
    this.seek = { time };
    this.time = time;
  }

  /**
   * @param report - Where the renderer's weather stood when it reported, or null while none plays.
   */
  @BoundAction()
  public noteReport(report: Nullable<IRendererWeatherReport>): void {
    if (report && report.time !== this.time) {
      this.time = report.time;
    }
  }

  /** Forgets the level's weather; the source and how it plays are the viewer's and stay. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.sessionId = null;
    this.description = null;
    this.cycle = null;
    this.playable = null;
    this.failure = null;
    this.seek = null;
    this.time = LEVEL_WEATHER_NOON;
  }
}
