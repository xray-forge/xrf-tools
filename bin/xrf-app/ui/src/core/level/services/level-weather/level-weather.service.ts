import { EventBus, inject, Injectable, OnDeactivation, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, comparer, Computed, flowResult, reaction, RefObservable } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { call, cancelFlows, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionPartResource } from "@xrf/ipc/session";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import {
  EnvironmentCycleEntry,
  LEVEL_ENVIRONMENT_PART,
  LEVEL_WEATHER_PART,
  LevelWeatherCycle,
  LevelWeatherDescription,
  SelectedLevelDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngine, XrayEngine } from "@xrf/ipc/types/xrf-engine-target";
import { EWeatherCycleKind, WeatherDescriptor } from "@xrf/ipc/types/xrf-environment";
import {
  EWorldWeatherPlay,
  EWorldWeatherTransition,
  WorldWeatherPlay,
  WorldWeatherReport,
} from "@xrf/ipc/types/xrf-world";
import { Logger } from "@xrf/logging";
import { Maybe, Nullable } from "@xrf/types";

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
import { ILevelWeatherHold } from "@/core/level/lib/weather/level-weather-hold";
import {
  ILevelWeatherMemory,
  readLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load";

/**
 * The open level's weather: what it plays, how, and where the renderer's clock stands. The renderer reads the cycle
 * and plays it, or the keyframe set by hand, by itself; this only says which and how, hears the time back, and
 * remembers it per level. The keyframe set by hand is also what lights a level whose weather does not play.
 *
 * What the level's weather is - its cycles, effects and textures - is document data, a part of the level read and read
 * again by its own resource; what plays and how is this service's view of it.
 */
@Injectable()
export class LevelWeatherService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The open level's weather as its part holds it, read again once its environment or what it plays with moves. */
  private readonly described: SessionPartResource<LevelWeatherDescription>;

  /**
   * The cycle played: loading while another is read, the one played held meanwhile; failed where the one asked for
   * does not read.
   */
  @RefObservable()
  public cycle: AsyncState<LevelWeatherCycle> = AsyncState.idle();

  /** The keyframe set by hand, or null before one was. */
  @RefObservable()
  public manual: Nullable<ILevelManualWeather> = null;

  /** What the keyframe set by hand was seeded from, keeping every key it does not set; null for none. */
  @RefObservable()
  public base: Nullable<WeatherDescriptor> = null;

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
  /** The cycle the level was last played with, kept until one plays so a level left early forgets nothing. */
  private remembered: string = "";
  /** The cycle last asked for, which `reading` names while it is read. */
  @RefObservable()
  private requested: Nullable<string> = null;
  /** Stops following the open level and its weather, while provisioned. */
  private unfollow: Array<() => void> = [];

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    eventBus: EventBus = inject(EventBus)
  ) {
    this.described = new SessionPartResource(
      eventBus,
      [LEVEL_ENVIRONMENT_PART, LEVEL_WEATHER_PART],
      levelsCommands.describeWeather
    );
  }

  /** The open level's weather: idle with none open, loading while it is read, failed where it does not read. */
  @Computed()
  public get description(): AsyncState<LevelWeatherDescription> {
    return this.described.state;
  }

  /**
   * What the renderer plays: the keyframe set by hand while it lights the level, the cycle otherwise, and the keyframe
   * again where no cycle plays; null while neither was handed over.
   */
  @Computed()
  public get weather(): Nullable<WorldWeatherPlay> {
    const manual: Nullable<WorldWeatherPlay> = this.isManualPlayed
      ? {
          keyframe: toLevelManualDescriptor(this.manual ?? DEFAULT_LEVEL_MANUAL_WEATHER, 0, this.base),
          kind: EWorldWeatherPlay.KEYFRAME,
        }
      : null;

    if (this.source === ELevelWeatherSource.MANUAL && manual) {
      return manual;
    }

    return this.cycle.value ? { kind: EWorldWeatherPlay.CYCLE, name: this.cycle.value.name } : manual;
  }

  /**
   * Why the level's weather, or the cycle last asked for, does not play; null while both do. A weather that failed to
   * be read again plays on as it was read, so says nothing.
   */
  @Computed()
  public get failure(): Nullable<string> {
    const description: Nullable<Error> = this.description.value ? null : this.description.error;

    return (description ?? this.cycle.error)?.message ?? null;
  }

  /** The cycle being read, or null while none is. */
  @Computed()
  public get reading(): Nullable<string> {
    return this.cycle.isLoading ? this.requested : null;
  }

  /** The engine the weather is read for: the open level's, Vanilla with none open. */
  @Computed()
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
  @Computed()
  public get isManual(): boolean {
    return this.source === ELevelWeatherSource.MANUAL || !this.cycle.value;
  }

  /**
   * The level's `weathers` where it leads to no cycle its game has, as a key its game's scripts read their own way does:
   * the viewer then plays the game's first cycle. Null while the level offers one, or before its weather is read.
   */
  @Computed()
  public get unfollowed(): Nullable<string> {
    const description: Nullable<LevelWeatherDescription> = this.description.value;

    return description && description.offered.length === 0 ? description.weather.key : null;
  }

  /** The level open, as its loader holds it, which the engine is named by. */
  private get selected(): Nullable<SessionSnapshot<SelectedLevelDescription>> {
    return this.loadService.level.value?.selected ?? null;
  }

  /**
   * Opens each level's weather as the level opens, whether or not a viewport plays it, and reads the cycle played
   * again each time the weather it is played from is read again.
   */
  @OnProvision()
  public onProvision(): void {
    this.unfollow = [
      this.loadService.followLevel((selected) => void this.open(selected)),
      reaction(
        () => this.described.snapshot,
        (
          read: Nullable<SessionSnapshot<LevelWeatherDescription>>,
          previous: Maybe<Nullable<SessionSnapshot<LevelWeatherDescription>>>
        ) => {
          if (read && previous && read.sessionId === previous.sessionId && read.revision > previous.revision) {
            void this.rereadCycle(read.sessionId);
          }
        }
      ),
    ];
  }

  @OnDeprovision()
  public onDeprovision(): void {
    this.unfollow.forEach((stop: () => void) => stop());
    this.unfollow = [];
  }

  /**
   * Remembers how the level's weather played, then forgets it all, for the next level to start from its own: every read
   * of the level's is cancelled, so none lands on the next.
   */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    cancelFlows(this);
    this.persist();
    this.sessionId = null;
    this.memoryKey = null;
    this.remembered = "";
    this.described.detach();
    this.cycle = AsyncState.idle();
    this.requested = null;
    this.manual = null;
    this.base = null;
    this.seed = null;
    this.isManualPlayed = false;
    this.transition = EWorldWeatherTransition.CUT;
    this.report = null;
    this.source = ELevelWeatherSource.WEATHER;
    this.control = DEFAULT_LEVEL_WEATHER_CONTROL;
    this.sunShafts = DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS;
    this.time = LEVEL_WEATHER_NOON;
  }

  /**
   * Plays another cycle from where the clock stands, faded into from what is shown; picking one is asking to see it,
   * so a level lit by hand is lit by the weather again once it is read. A cycle picked meanwhile, or another level
   * opened, cancels it before it shows or lights anything.
   *
   * @param name - The cycle's name.
   */
  @LatestFlow("cycle")
  public *selectCycle(name: string): TFlow {
    const sessionId: Nullable<string> = this.sessionId;

    if (!sessionId) {
      return;
    }

    try {
      if (this.cycle.value?.name === name) {
        this.keepCycle(name);
      } else {
        yield* this.play(sessionId, name);
      }

      this.setSource(ELevelWeatherSource.WEATHER);
      this.persist();
    } catch (error: unknown) {
      this.fail(error);
    }
  }

  /**
   * Holds the clock still for something borrowing the weather, such as a capture, without remembering it for the
   * level; `release` plays what was held again.
   *
   * @returns What played until now.
   */
  @BoundAction()
  public hold(): ILevelWeatherHold {
    const held: ILevelWeatherHold = {
      cycle: this.cycle.value?.name ?? this.remembered,
      source: this.source,
      control: this.control,
      time: this.time,
    };

    this.control = { ...this.control, isPaused: true };

    return held;
  }

  /**
   * Plays a cycle at once, cut in rather than faded, and lit by the weather: what a level map is captured under, while
   * the weather is held. Nothing of it is remembered for the level; `release` plays what was held again.
   *
   * @param name - The cycle's name.
   */
  @LatestFlow("cycle")
  public *cutTo(name: string): TFlow {
    const sessionId: Nullable<string> = this.sessionId;

    if (!sessionId) {
      return;
    }

    if (this.cycle.value?.name !== name) {
      yield* this.play(sessionId, name, EWorldWeatherTransition.CUT);
    }

    const shown: Nullable<WorldWeatherPlay> = this.weather;

    this.source = ELevelWeatherSource.WEATHER;
    this.noteShown(shown, EWorldWeatherTransition.CUT);
  }

  /**
   * Plays what `hold` kept again, cut in: its cycle, what lit the level and its clock's control. Where its clock stood
   * is the caller's to seek, as the renderer owns the clock.
   *
   * @param held - What played when the weather was borrowed.
   */
  @LatestFlow("cycle")
  public *release(held: ILevelWeatherHold): TFlow {
    const sessionId: Nullable<string> = this.sessionId;

    if (!sessionId) {
      return;
    }

    try {
      if (held.cycle && this.cycle.value?.name !== held.cycle) {
        yield* this.play(sessionId, held.cycle, EWorldWeatherTransition.CUT);
      }
    } catch (error: unknown) {
      this.fail(error);
    }

    const shown: Nullable<WorldWeatherPlay> = this.weather;

    this.source = held.source;
    this.control = held.control;
    this.noteShown(shown, EWorldWeatherTransition.CUT);
    this.persist();
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
   * Sets keys of the keyframe set by hand back to what the weather showed when it was seeded, noon of `default_clear`
   * where it was seeded from nothing; nothing while the weather lights the level, as its keys are the weather's.
   *
   * @param keys - The keys set back.
   */
  @BoundAction()
  public resetManual(keys: ReadonlyArray<keyof ILevelManualWeather>): void {
    if (!this.isManual) {
      return;
    }

    const weather: ILevelManualWeather = this.seed?.weather ?? DEFAULT_LEVEL_MANUAL_WEATHER;

    this.editManual(Object.fromEntries(keys.map((key) => [key, weather[key]])));
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

  @BoundAction()
  public setSunShafts(sunShafts: ILevelSunShaftsOptions): void {
    this.sunShafts = sunShafts;
    this.persist();
  }

  /**
   * Opens a level's weather, once per level: how it was last played, then the weather itself, read and played.
   *
   * @param selected - The level open now, or null for none.
   */
  private async open(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): Promise<void> {
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

    if (memory) {
      this.restore(memory);
    }

    await flowResult(this.read(selected, memory?.cycle ?? ""));
  }

  /**
   * Reads the level's weather and plays the cycle it was last played with, or the first it offers, or the game's first
   * where it offers none; or the keyframe set by hand, where that is what it was lit by, or where no cycle plays.
   *
   * On the cycle lane, as every play is: a cycle picked meanwhile is the last word, and another level opened meanwhile
   * cancels it with the rest of the level's work.
   *
   * @param selected - The level open.
   * @param remembered - The cycle it was last played with, empty for none.
   */
  @LatestFlow("cycle")
  private *read(selected: SessionSnapshot<SelectedLevelDescription>, remembered: string): TFlow {
    yield* call(this.described.attach(selected.sessionId));

    const description: Nullable<LevelWeatherDescription> = this.described.value;

    // The keyframe set by hand is what lights a level whose weather does not read.
    if (!description) {
      this.log.warn("The level's weather is not played:", this.described.error?.message);
      this.playManual(EWorldWeatherTransition.CUT);

      return;
    }

    try {
      // Lit by hand, the keyframe is handed over first, so the cycle is never shown before it.
      if (this.source === ELevelWeatherSource.MANUAL) {
        this.playManual(EWorldWeatherTransition.CUT);
      }

      // A level whose `weathers` its game's scripts read their own way offers nothing; the game's own cycles stand in.
      const first: Maybe<string> =
        description.offered[0]?.name ??
        description.cycles.find((it: EnvironmentCycleEntry) => it.kind === EWeatherCycleKind.CYCLE)?.name;

      // A remembered cycle the game no longer has gives way to the level's own.
      if (remembered && (yield* this.playRemembered(selected.sessionId, remembered))) {
        return;
      }

      if (first) {
        yield* this.play(selected.sessionId, first);
      } else if (this.source !== ELevelWeatherSource.MANUAL) {
        this.playManual(EWorldWeatherTransition.CUT);
      }
    } catch (error: unknown) {
      this.fail(error);

      // The keyframe set by hand is what lights a level whose weather does not play.
      this.playManual(EWorldWeatherTransition.CUT);
    }
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
    this.base = memory.base;
    this.seed = memory.seed;
    this.sunShafts = memory.sunShafts;
    this.remembered = memory.cycle;
  }

  /**
   * A keyframe set by hand from what is shown, noon of `default_clear` where the weather shows nothing yet; what is shown
   * is kept whole as its base.
   */
  private seedManual(): ILevelManualWeather {
    const current = this.report?.current;
    const weather: ILevelManualWeather = current ? toLevelManualWeather(current) : DEFAULT_LEVEL_MANUAL_WEATHER;

    this.base = current ?? null;
    this.seed = current && this.cycle.value ? { cycle: this.cycle.value.name, time: this.time, weather } : null;

    return weather;
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
   * @param sessionId - The level it plays in.
   * @param name - The cycle's name.
   * @returns Whether it plays.
   */
  private *playRemembered(sessionId: string, name: string): TFlow<boolean> {
    try {
      yield* this.play(sessionId, name);

      return true;
    } catch (error: unknown) {
      this.log.warn(`The remembered cycle '${name}' is not played:`, transformError(error).message);

      return false;
    }
  }

  /**
   * Asks again for the cycle held; the pick asking it has cancelled any read of another one on the cycle lane.
   *
   * @param name - The held cycle's name.
   */
  @BoundAction()
  private keepCycle(name: string): void {
    this.requested = name;

    if (this.cycle.isLoading) {
      this.cycle = this.cycle.asReady();
    }
  }

  /**
   * Reads a cycle, for what the panels show of it, then plays it, inside the cycle lane's flow that asked for it.
   *
   * @param sessionId - The level it plays in.
   * @param name - The cycle's name.
   * @param transition - How it takes over from what is shown.
   */
  private *play(
    sessionId: string,
    name: string,
    transition: EWorldWeatherTransition = EWorldWeatherTransition.FADE
  ): TFlow {
    this.requested = name;
    this.cycle = this.cycle.asLoading();

    try {
      const cycle: LevelWeatherCycle = yield* this.readCycle(sessionId, name);
      const shown: Nullable<WorldWeatherPlay> = this.weather;

      this.cycle = this.cycle.asReady(cycle);
      this.noteShown(shown, transition);
    } finally {
      // No longer being read, whether it failed, which the caller says is worth stating or not, or a newer play
      // cancelled it: what played before stays played.
      if (this.cycle.isLoading) {
        this.cycle = this.cycle.value ? this.cycle.asReady() : this.cycle.asIdle();
      }
    }
  }

  /**
   * Reads a cycle as the level's weather offers it, or from its game otherwise; one read before the weather shown is
   * read again, from what that weather was read from.
   *
   * @param sessionId - The level it plays in.
   * @param name - The cycle's name.
   * @returns The cycle.
   */
  private *readCycle(sessionId: string, name: string): TFlow<LevelWeatherCycle> {
    const offered: Maybe<LevelWeatherCycle> = this.description.value?.offered.find(
      (it: LevelWeatherCycle) => it.name === name
    );

    if (offered) {
      return offered;
    }

    const read: SessionSnapshot<LevelWeatherCycle> = yield* call(
      levelsCommands.readLevelCycle(sessionId, { kind: EWeatherCycleKind.CYCLE, name })
    );

    const shown: Maybe<number> = this.described.snapshot?.revision;

    return read.revision < (shown ?? read.revision) ? yield* this.readCycle(sessionId, name) : read.value;
  }

  /**
   * Reads the cycle played again from the level's weather read again, keeping what plays and how; a lane of its own,
   * so it never cancels a cycle being picked, and drops its answer where another cycle played meanwhile.
   *
   * @param sessionId - The level open.
   */
  @LatestFlow()
  private *rereadCycle(sessionId: string): TFlow {
    const name: Maybe<string> = this.cycle.value?.name;

    if (!name) {
      return;
    }

    try {
      const cycle: LevelWeatherCycle = yield* this.readCycle(sessionId, name);

      if (this.cycle.value?.name === name) {
        this.cycle = this.cycle.map(() => cycle);
      }
    } catch (error: unknown) {
      this.log.warn("The level's cycle played was not read again:", transformError(error).message);
    }
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
   * Notes why the cycle asked for does not play, inside the flow that asked; why the level's weather does not read is
   * its resource's to say.
   *
   * @param error - Why it failed.
   */
  private fail(error: unknown): void {
    const failure: Error = transformError(error);

    this.log.warn("The level's weather is not played:", failure.message);
    this.cycle = this.cycle.asFailed(failure);
  }

  /** Remembers how the open level's weather plays, once a cycle or the keyframe set by hand does. */
  private persist(): void {
    const cycle: string = this.cycle.value?.name ?? this.remembered;

    if (this.memoryKey && (cycle || this.manual)) {
      writeLevelWeatherMemory(this.memoryKey, {
        control: this.control,
        cycle,
        base: this.base,
        manual: this.manual,
        seed: this.seed,
        source: this.source,
        sunShafts: this.sunShafts,
        time: this.time,
      });
    }
  }
}
