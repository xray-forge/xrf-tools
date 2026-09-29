import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable, runInAction } from "@wirestate/mobx";
import {
  ERendererWeatherTransition,
  IRendererWeather,
  IRendererWeatherKeyframe,
  IRendererWeatherReport,
} from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import {
  LevelTextureReference,
  LevelWeatherCycle,
  LevelWeatherDescription,
  SelectedLevelDescription,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EWeatherCycleKind } from "@/core/ipc/types/xrf-environment";
import { toLevelManualRendererWeather } from "@/core/level/lib/weather/level-manual-renderer-weather";
import {
  DEFAULT_LEVEL_MANUAL_WEATHER,
  ILevelManualWeather,
  listLevelManualWeatherTextures,
  toLevelManualWeather,
} from "@/core/level/lib/weather/level-manual-weather";
import { toLevelRendererWeather } from "@/core/level/lib/weather/level-renderer-weather";
import {
  DEFAULT_LEVEL_WEATHER_CONTROL,
  ILevelWeatherControl,
  LEVEL_WEATHER_NOON,
  toLevelWeatherFactor,
} from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherEffectRequest } from "@/core/level/lib/weather/level-weather-effect-request";
import {
  ILevelWeatherMemory,
  readLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { ILevelWeatherSeek } from "@/core/level/lib/weather/level-weather-seek";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/**
 * The open level's weather: what it plays, how, and where the renderer's clock stands. The renderer plays the cycle,
 * or the keyframe set by hand, by itself; this only says which and how, hears the time back, and remembers it per
 * level. The keyframe set by hand is also what lights a level whose weather does not play.
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

  /** The keyframe set by hand, or null before one was. */
  @RefObservable()
  public manual: Nullable<ILevelManualWeather> = null;

  /** What the keyframe set by hand was seeded from, or null for none the level played. */
  @RefObservable()
  public seed: Nullable<ILevelWeatherSeed> = null;

  /** The keyframe set by hand as the renderer plays it, or null while it is being built or there is none. */
  @RefObservable()
  public manualPlayable: Nullable<IRendererWeather> = null;

  /** How the weather the renderer is handed next takes over from what it shows. */
  @RefObservable()
  public transition: ERendererWeatherTransition = ERendererWeatherTransition.CUT;

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
  /** What each texture the keyframe set by hand named came to, so an edit that names none new asks for nothing. */
  private readonly located: Map<string, LevelTextureReference> = new Map();
  /** The cycle the level was last played with, kept until one plays so a level left early forgets nothing. */
  private remembered: string = "";
  /** Bumped by every build of the keyframe set by hand, so only the latest is played. */
  private build: number = 0;

  public constructor(private readonly settingsService: SettingsService = inject(SettingsService)) {}

  /**
   * What the renderer plays: the keyframe set by hand once built while it lights the level, the cycle otherwise, and
   * the keyframe again where no cycle plays; null while neither is ready.
   */
  public get weather(): Nullable<IRendererWeather> {
    if (this.source === ELevelWeatherSource.MANUAL && this.manualPlayable) {
      return this.manualPlayable;
    }

    return this.playable ?? this.manualPlayable;
  }

  /** The engine the weather is read for: the level's, or the setting's where its weather does not read. */
  public get engine(): XrayEngine {
    return this.description?.engine ?? this.settingsService.engine;
  }

  /** The keyframe on screen as one set by hand: the one set, or the weather's mix, which a first edit seeds it from. */
  public get shown(): ILevelManualWeather {
    const current: Maybe<IRendererWeatherKeyframe> = this.report?.current;

    return !this.isManual && current ? toLevelManualWeather(current) : (this.manual ?? DEFAULT_LEVEL_MANUAL_WEATHER);
  }

  /** Whether the keyframe set by hand lights the level, by choice or because no cycle plays. */
  public get isManual(): boolean {
    return this.source === ELevelWeatherSource.MANUAL || !this.playable;
  }

  /**
   * Reads a level's weather and plays the cycle it was last played with, or the first it offers, once per level; or
   * the keyframe set by hand, where that is what it was lit by, or where its weather does not play.
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
        this.manual = memory.manual;
        this.seed = memory.seed;
      });
      this.remembered = memory.cycle;
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

      // Lit by hand, the keyframe is built first, so the cycle is never shown before it.
      if (this.source === ELevelWeatherSource.MANUAL) {
        await this.buildManual(ERendererWeatherTransition.CUT);
      }

      const first: Maybe<LevelWeatherCycle> = description.offered[0];

      if (!first) {
        throw new Error("The level's weathers resolve to no cycle the game has");
      }

      // A remembered cycle the game no longer has gives way to the level's own.
      if (!memory?.cycle || !(await this.playRemembered(selected, memory.cycle))) {
        await this.play(selected, first.name);
      }
    } catch (error: unknown) {
      this.fail(selected.sessionId, error);
      // The keyframe set by hand is what lights a level whose weather does not play.
      await this.buildManual(ERendererWeatherTransition.CUT);
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
      if (this.cycle?.name !== name) {
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

    this.transition = ERendererWeatherTransition.FADE;
    this.source = source;

    if (source === ELevelWeatherSource.MANUAL) {
      this.manual ??= this.seedManual();
      void this.buildManual(ERendererWeatherTransition.FADE);
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

    this.manual = { ...(this.manual ?? DEFAULT_LEVEL_MANUAL_WEATHER), ...patch };
    void this.buildManual(ERendererWeatherTransition.EASE);
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
    this.remembered = "";
    this.located.clear();
    this.build += 1;
    this.description = null;
    this.cycle = null;
    this.playable = null;
    this.manual = null;
    this.seed = null;
    this.manualPlayable = null;
    this.transition = ERendererWeatherTransition.CUT;
    this.failure = null;
    this.reading = null;
    this.seek = null;
    this.effect = null;
    this.report = null;
    this.source = ELevelWeatherSource.WEATHER;
    this.control = DEFAULT_LEVEL_WEATHER_CONTROL;
    this.time = LEVEL_WEATHER_NOON;
  }

  /** A keyframe set by hand from what is shown, noon of `default_clear` where the weather shows nothing yet. */
  private seedManual(): ILevelManualWeather {
    const current = this.report?.current;

    this.seed = current && this.cycle ? { cycle: this.cycle.name, time: this.time } : null;

    return current ? toLevelManualWeather(current) : DEFAULT_LEVEL_MANUAL_WEATHER;
  }

  /**
   * Builds what the renderer plays of the keyframe set by hand, its textures resolved as the level resolves its own,
   * only the latest build played.
   *
   * @param transition - How it takes over from what is shown.
   */
  private async buildManual(transition: ERendererWeatherTransition): Promise<void> {
    const { selected } = this;
    const build: number = ++this.build;

    runInAction(() => {
      this.manual ??= DEFAULT_LEVEL_MANUAL_WEATHER;
    });

    const manual: Nullable<ILevelManualWeather> = this.manual;

    if (!selected || !manual) {
      return;
    }

    try {
      const references: Array<string> = listLevelManualWeatherTextures(manual);
      const unknown: Array<string> = references.filter((it: string) => !this.located.has(it));

      if (unknown.length) {
        const { value } = await levelsCommands.resolveLevelTextures(selected.sessionId, unknown);

        // Found in the level asked about: another level open since finds its textures in its own roots.
        if (this.sessionId !== selected.sessionId) {
          return;
        }

        value.forEach((it: LevelTextureReference) => this.located.set(it.reference, it));
      }

      const weather: IRendererWeather = await toLevelManualRendererWeather({
        description: this.description,
        engine: this.engine,
        located: references.flatMap((it: string) => this.located.get(it) ?? []),
        manual,
        roots: selected.value.roots,
      });

      if (build === this.build && this.sessionId === selected.sessionId) {
        runInAction(() => {
          const shown: Nullable<IRendererWeather> = this.weather;

          this.manualPlayable = weather;
          this.noteShown(shown, transition);
        });
      }
    } catch (error: unknown) {
      this.log.warn("The keyframe set by hand is not played:", transformError(error).message);
    }
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
   * Reads a cycle and asks where its skies are fetched from, then plays it, for a level still open.
   *
   * @param selected - The level it plays in.
   * @param name - The cycle's name.
   */
  private async play(selected: SessionSnapshot<SelectedLevelDescription>, name: string): Promise<void> {
    runInAction(() => {
      this.reading = name;
    });

    const cycle: LevelWeatherCycle =
      this.description?.offered.find((it: LevelWeatherCycle) => it.name === name) ??
      (await levelsCommands.readLevelCycle(selected.sessionId, { kind: EWeatherCycleKind.CYCLE, name })).value;
    const description: Nullable<LevelWeatherDescription> = this.description;

    if (!description) {
      return;
    }

    const playable: IRendererWeather = await toLevelRendererWeather({
      cycle,
      description,
      roots: selected.value.roots,
    });

    if (this.sessionId !== selected.sessionId || this.reading !== name) {
      return;
    }

    runInAction(() => {
      const shown: Nullable<IRendererWeather> = this.weather;

      this.cycle = cycle;
      this.playable = playable;
      this.failure = null;
      this.reading = null;
      this.noteShown(shown, ERendererWeatherTransition.FADE);
    });
  }

  /**
   * Says how what the renderer plays now takes over, where it changed: the first weather the level shows cuts in.
   *
   * @param shown - What it played before.
   * @param transition - How anything after the first takes over.
   */
  private noteShown(shown: Nullable<IRendererWeather>, transition: ERendererWeatherTransition): void {
    if (this.weather !== shown) {
      this.transition = shown ? transition : ERendererWeatherTransition.CUT;
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

  /** Remembers how the open level's weather plays, once a cycle or the keyframe set by hand does. */
  private persist(): void {
    const cycle: string = this.cycle?.name ?? this.remembered;

    if (this.memoryKey && (cycle || this.manual)) {
      writeLevelWeatherMemory(this.memoryKey, {
        control: this.control,
        cycle,
        manual: this.manual,
        seed: this.seed,
        source: this.source,
        time: this.time,
      });
    }
  }
}
