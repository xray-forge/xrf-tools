import { inject, Injectable, OnDeactivation, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, comparer, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import {
  EnvironmentCycleEntry,
  LevelWeatherCycle,
  LevelWeatherDescription,
  SelectedLevelDescription,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { EXrayEngine, XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EWeatherCycleKind, WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import {
  EWorldWeatherPlay,
  EWorldWeatherTransition,
  WorldWeatherPlay,
  WorldWeatherReport,
} from "@/core/ipc/types/xrf-world";
import {
  DEFAULT_LEVEL_MANUAL_WEATHER,
  ILevelManualWeather,
  toLevelManualDescriptor,
  toLevelManualWeather,
} from "@/core/level/lib/weather/level-manual-weather";
import {
  DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
  ILevelSunShaftsOptions,
} from "@/core/level/lib/weather/level-sun-shafts-options";
import {
  DEFAULT_LEVEL_WEATHER_CONTROL,
  ILevelWeatherControl,
  LEVEL_WEATHER_NOON,
  toLevelWeatherFactor,
} from "@/core/level/lib/weather/level-weather-control";
import {
  ILevelWeatherMemory,
  readLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { Logger } from "@/lib/logging";

/**
 * The open level's weather: what it plays, how, and where the renderer's clock stands. The renderer reads the cycle
 * and plays it, or the keyframe set by hand, by itself; this only says which and how, hears the time back, and
 * remembers it per level. The keyframe set by hand is also what lights a level whose weather does not play.
 */
@Injectable()
export class LevelWeatherService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The open level's weather: idle with none open, loading while it is read, failed where it does not read. */
  @RefObservable()
  public description: AsyncState<LevelWeatherDescription> = AsyncState.idle();

  /**
   * The cycle played: loading while another is read, the one played held meanwhile; failed where the one asked for
   * does not read.
   */
  @RefObservable()
  public cycle: AsyncState<LevelWeatherCycle> = AsyncState.idle();

  /** The keyframe set by hand, or null before one was. */
  @RefObservable()
  public manual: Nullable<ILevelManualWeather> = null;

  /** What the keyframe set by hand was seeded from, or null for none the level played. */
  @RefObservable()
  public seed: Nullable<ILevelWeatherSeed> = null;

  /** Whether the keyframe set by hand was handed to the renderer, as it is once it lights the level or none plays. */
  @RefObservable()
  public isManualPlayed: boolean = false;

  /** How the weather the renderer is handed next takes over from what it shows. */
  @RefObservable()
  public transition: EWorldWeatherTransition = EWorldWeatherTransition.CUT;

  @RefObservable()
  public source: ELevelWeatherSource = ELevelWeatherSource.WEATHER;

  @RefObservable()
  public control: ILevelWeatherControl = DEFAULT_LEVEL_WEATHER_CONTROL;

  /** How the level's sun shafts step, and the floor under their density: remembered with its weather. */
  @RefObservable()
  public sunShafts: ILevelSunShaftsOptions = DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS;

  /** Where the clock stands, in seconds since midnight: sent to, or heard back from the renderer. */
  @RefObservable()
  public time: number = LEVEL_WEATHER_NOON;

  /** Where the renderer's weather stood when it last reported, or null before it has. */
  @RefObservable()
  public report: Nullable<WorldWeatherReport> = null;

  /** The level whose weather is held or being read. */
  private sessionId: Nullable<string> = null;
  /** What the level's weather is remembered under. */
  private memoryKey: Nullable<string> = null;
  /** The level open, which a cycle is read against. */
  private selected: Nullable<SessionSnapshot<SelectedLevelDescription>> = null;
  /** The cycle the level was last played with, kept until one plays so a level left early forgets nothing. */
  private remembered: string = "";
  /** The cycle last asked for, which a read of another one since gives way to. */
  @RefObservable()
  private requested: Nullable<string> = null;
  /** Stops following the open level, while provisioned. */
  private unfollow: Nullable<() => void> = null;

  public constructor(private readonly loadService: LevelLoadService = inject(LevelLoadService)) {}

  /** Opens each level's weather as the level opens, whether or not a viewport plays it. */
  @OnProvision()
  public onProvision(): void {
    this.unfollow = this.loadService.followLevel((selected) => void this.open(selected));
  }

  @OnDeprovision()
  public onDeprovision(): void {
    this.unfollow?.();
    this.unfollow = null;
  }

  /**
   * What the renderer plays: the keyframe set by hand while it lights the level, the cycle otherwise, and the keyframe
   * again where no cycle plays; null while neither was handed over.
   */
  public get weather(): Nullable<WorldWeatherPlay> {
    const manual: Nullable<WorldWeatherPlay> = this.isManualPlayed
      ? {
          keyframe: toLevelManualDescriptor(this.manual ?? DEFAULT_LEVEL_MANUAL_WEATHER, 0),
          kind: EWorldWeatherPlay.KEYFRAME,
        }
      : null;

    if (this.source === ELevelWeatherSource.MANUAL && manual) {
      return manual;
    }

    return this.cycle.value ? { kind: EWorldWeatherPlay.CYCLE, name: this.cycle.value.name } : manual;
  }

  /** Why the level's weather, or the cycle last asked for, does not play; null while both do. */
  public get failure(): Nullable<string> {
    return (this.description.error ?? this.cycle.error)?.message ?? null;
  }

  /** The cycle being read, or null while none is. */
  public get reading(): Nullable<string> {
    return this.cycle.isLoading ? this.requested : null;
  }

  /** The engine the weather is read for: the open level's, Vanilla with none open. */
  public get engine(): XrayEngine {
    return this.selected?.value.engine.engine ?? EXrayEngine.VANILLA;
  }

  /** The keyframe on screen as one set by hand: the one set, or the weather's mix, which a first edit seeds it from. */
  @Computed()
  public get shown(): ILevelManualWeather {
    const current: Maybe<WeatherDescriptor> = this.isManual ? undefined : this.report?.current;

    return current ? toLevelManualWeather(current) : (this.manual ?? DEFAULT_LEVEL_MANUAL_WEATHER);
  }

  /** Whether the keyframe set by hand lights the level, by choice or because no cycle plays. */
  public get isManual(): boolean {
    return this.source === ELevelWeatherSource.MANUAL || !this.cycle.value;
  }

  /**
   * Opens a level's weather, once per level: how it was last played, then the weather itself, read and played.
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
      this.restore(memory);
    }

    await this.read(selected, memory?.cycle ?? "");
  }

  /**
   * The level's `weathers` where it leads to no cycle its game has, as a key its game's scripts read their own way does:
   * the viewer then plays the game's first cycle. Null while the level offers one, or before its weather is read.
   */
  public get unfollowed(): Nullable<string> {
    const description: Nullable<LevelWeatherDescription> = this.description.value;

    return description && description.offered.length === 0 ? description.weather.key : null;
  }

  /**
   * Reads the level's weather and plays the cycle it was last played with, or the first it offers, or the game's first
   * where it offers none; or the keyframe set by hand, where that is what it was lit by, or where no cycle plays.
   *
   * @param selected - The level open.
   * @param remembered - The cycle it was last played with, empty for none.
   */
  private async read(selected: SessionSnapshot<SelectedLevelDescription>, remembered: string): Promise<void> {
    runInAction(() => {
      this.description = this.description.asLoading();
    });

    try {
      const { value: description }: SessionSnapshot<LevelWeatherDescription> = await levelsCommands.readLevelWeather(
        selected.sessionId
      );

      if (this.sessionId !== selected.sessionId) {
        return;
      }

      runInAction(() => {
        this.description = this.description.asReady(description);
      });

      // Lit by hand, the keyframe is handed over first, so the cycle is never shown before it.
      if (this.source === ELevelWeatherSource.MANUAL) {
        this.playManual(EWorldWeatherTransition.CUT);
      }

      // A level whose `weathers` its game's scripts read their own way offers nothing; the game's own cycles stand in.
      const first: Maybe<string> =
        description.offered[0]?.name ??
        description.cycles.find((it: EnvironmentCycleEntry) => it.kind === EWeatherCycleKind.CYCLE)?.name;

      // A remembered cycle the game no longer has gives way to the level's own.
      if (remembered && (await this.playRemembered(selected, remembered))) {
        return;
      }

      if (first) {
        await this.play(selected, first);
      } else if (this.source !== ELevelWeatherSource.MANUAL) {
        this.playManual(EWorldWeatherTransition.CUT);
      }
    } catch (error: unknown) {
      this.fail(selected.sessionId, error);

      // The keyframe set by hand is what lights a level whose weather does not play, unless another level opened since.
      if (this.sessionId === selected.sessionId) {
        this.playManual(EWorldWeatherTransition.CUT);
      }
    }
  }

  /**
   * Plays another cycle from where the clock stands, faded into from what is shown; picking one is asking to see it,
   * so a level lit by hand is lit by the weather again once it is read.
   *
   * @param name - The cycle's name.
   */
  public async selectCycle(name: string): Promise<void> {
    const sessionId: Nullable<string> = this.sessionId;
    const selected: Nullable<SessionSnapshot<SelectedLevelDescription>> = this.selected;

    if (!sessionId || !selected) {
      return;
    }

    try {
      if (this.cycle.value?.name !== name) {
        await this.play(selected, name);
      }

      this.setSource(ELevelWeatherSource.WEATHER);
      this.persist();
    } catch (error: unknown) {
      this.fail(sessionId, error);
    }
  }

  /**
   * Lights the level by its weather or by the keyframe set by hand, faded into; the keyframe is seeded from what is
   * shown the first time it is asked for.
   *
   * @param source - What lights it from now on.
   */
  @BoundAction()
  public setSource(source: ELevelWeatherSource): void {
    if (source === this.source) {
      return;
    }

    this.transition = EWorldWeatherTransition.FADE;
    this.source = source;

    if (source === ELevelWeatherSource.MANUAL) {
      this.manual ??= this.seedManual();
      this.playManual(EWorldWeatherTransition.FADE);
    }

    this.persist();
  }

  /**
   * Changes the keyframe set by hand, which lights the level from now on: one edited while the weather lights it is
   * seeded from what is shown first, so only the change is seen, and eased into.
   *
   * @param patch - The keys changed.
   */
  @BoundAction()
  public editManual(patch: Partial<ILevelManualWeather>): void {
    if (!this.isManual) {
      this.manual = this.seedManual();
      this.source = ELevelWeatherSource.MANUAL;
    }

    const shown: Nullable<WorldWeatherPlay> = this.weather;

    this.manual = { ...(this.manual ?? DEFAULT_LEVEL_MANUAL_WEATHER), ...patch };
    this.isManualPlayed = true;
    this.noteShown(shown, EWorldWeatherTransition.EASE);
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

  /** Remembers how the level's weather played, then forgets it all, for the next level to start from its own. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.persist();
    this.sessionId = null;
    this.memoryKey = null;
    this.selected = null;
    this.remembered = "";
    this.description = AsyncState.idle();
    this.cycle = AsyncState.idle();
    this.requested = null;
    this.manual = null;
    this.seed = null;
    this.isManualPlayed = false;
    this.transition = EWorldWeatherTransition.CUT;
    this.report = null;
    this.source = ELevelWeatherSource.WEATHER;
    this.control = DEFAULT_LEVEL_WEATHER_CONTROL;
    this.sunShafts = DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS;
    this.time = LEVEL_WEATHER_NOON;
  }

  @BoundAction()
  public setSunShafts(sunShafts: ILevelSunShaftsOptions): void {
    this.sunShafts = sunShafts;
    this.persist();
  }

  /**
   * @param memory - How the level's weather was last played, which it plays from again.
   */
  @BoundAction()
  private restore(memory: ILevelWeatherMemory): void {
    this.source = memory.source;
    this.control = memory.control;
    this.time = memory.time;
    this.manual = memory.manual;
    this.seed = memory.seed;
    this.sunShafts = memory.sunShafts;
    this.remembered = memory.cycle;
  }

  /** A keyframe set by hand from what is shown, noon of `default_clear` where the weather shows nothing yet. */
  private seedManual(): ILevelManualWeather {
    const current = this.report?.current;

    this.seed = current && this.cycle.value ? { cycle: this.cycle.value.name, time: this.time } : null;

    return current ? toLevelManualWeather(current) : DEFAULT_LEVEL_MANUAL_WEATHER;
  }

  /**
   * Hands the renderer the keyframe set by hand, the default one where none was set.
   *
   * @param transition - How it takes over from what is shown.
   */
  @BoundAction()
  private playManual(transition: EWorldWeatherTransition): void {
    const shown: Nullable<WorldWeatherPlay> = this.weather;

    this.manual ??= DEFAULT_LEVEL_MANUAL_WEATHER;
    this.isManualPlayed = true;
    this.noteShown(shown, transition);
  }

  /**
   * Plays the cycle a level was last played with, where it still reads.
   *
   * @param selected - The level it plays in.
   * @param name - The cycle's name.
   * @returns Whether it plays.
   */
  private async playRemembered(selected: SessionSnapshot<SelectedLevelDescription>, name: string): Promise<boolean> {
    try {
      await this.play(selected, name);

      return true;
    } catch (error: unknown) {
      this.log.warn(`The remembered cycle '${name}' is not played:`, transformError(error).message);

      return false;
    }
  }

  /**
   * Reads a cycle, for what the panels show of it, then plays it, for a level still open.
   *
   * @param selected - The level it plays in.
   * @param name - The cycle's name.
   */
  private async play(selected: SessionSnapshot<SelectedLevelDescription>, name: string): Promise<void> {
    runInAction(() => {
      this.requested = name;
      this.cycle = this.cycle.asLoading();
    });

    let cycle: LevelWeatherCycle;

    try {
      cycle =
        this.description.value?.offered.find((it: LevelWeatherCycle) => it.name === name) ??
        (await levelsCommands.readLevelCycle(selected.sessionId, { kind: EWeatherCycleKind.CYCLE, name })).value;
    } catch (error: unknown) {
      // A cycle asked for since decides what plays; this one's failure no longer says anything.
      if (this.requested !== name) {
        return;
      }

      // No longer being read: the caller says whether its failure is worth stating.
      runInAction(() => {
        this.cycle = this.cycle.value ? this.cycle.asReady() : this.cycle.asIdle();
      });

      throw error;
    }

    if (this.sessionId !== selected.sessionId || this.requested !== name) {
      return;
    }

    runInAction(() => {
      const shown: Nullable<WorldWeatherPlay> = this.weather;

      this.cycle = this.cycle.asReady(cycle);
      this.noteShown(shown, EWorldWeatherTransition.FADE);
    });
  }

  /**
   * Says how what the renderer plays now takes over, where it changed: the first weather the level shows cuts in.
   *
   * @param shown - What it played before.
   * @param transition - How anything after the first takes over.
   */
  private noteShown(shown: Nullable<WorldWeatherPlay>, transition: EWorldWeatherTransition): void {
    if (!comparer.structural(this.weather, shown)) {
      this.transition = shown ? transition : EWorldWeatherTransition.CUT;
    }
  }

  /**
   * Notes why the weather does not play: the level's weather where it never read, the cycle asked for otherwise.
   *
   * @param sessionId - The level the failed read belongs to.
   * @param error - Why it failed.
   */
  private fail(sessionId: string, error: unknown): void {
    const failure: Error = transformError(error);

    this.log.warn("The level's weather is not played:", failure.message);

    if (this.sessionId === sessionId) {
      runInAction(() => {
        if (this.description.isReady) {
          this.cycle = this.cycle.asFailed(failure);
        } else {
          this.description = this.description.asFailed(failure);
        }
      });
    }
  }

  /** Remembers how the open level's weather plays, once a cycle or the keyframe set by hand does. */
  private persist(): void {
    const cycle: string = this.cycle.value?.name ?? this.remembered;

    if (this.memoryKey && (cycle || this.manual)) {
      writeLevelWeatherMemory(this.memoryKey, {
        control: this.control,
        cycle,
        manual: this.manual,
        seed: this.seed,
        source: this.source,
        sunShafts: this.sunShafts,
        time: this.time,
      });
    }
  }
}
