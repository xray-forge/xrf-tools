import { clamp } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";

import { ERendererEngine } from "#/contract/renderer-engine";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IRendererWeatherReport } from "#/contract/weather/renderer-weather-report";
import { ERendererWeatherTransition } from "#/contract/weather/renderer-weather-transition";
import { RendererTextures } from "#/texture/renderer-textures";
import { toWeatherTimeOfDay } from "#/weather/weather-day";
import { IWeatherEffectTimeline, toWeatherEffectTimeline } from "#/weather/weather-effect-timeline";
import { WeatherFader } from "#/weather/weather-fader";
import { listWeatherTextures } from "#/weather/weather-held-textures";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { toWeatherLighting, toWeatherMixedKeyframe } from "#/weather/weather-lighting";
import { IWeatherMix } from "#/weather/weather-mix";
import { mixWeatherPair } from "#/weather/weather-mixer";
import { WeatherPair } from "#/weather/weather-pair";
import { EWeatherSun, TWeatherSun } from "#/weather/weather-sun";
import { WeatherTextures } from "#/weather/weather-textures";
import { WeatherThunder } from "#/weather/weather-thunder";
import { IWeatherThunderFlash } from "#/weather/weather-thunder-flash";
import { toThunderedLighting } from "#/weather/weather-thundered-lighting";

/** Seconds from midnight to noon. */
const NOON: number = 12 * 60 * 60;

/** How a weather plays until told otherwise: paused at noon, at the engine's own time factor. */
const DEFAULT_CONTROL: IRendererWeatherControl = {
  factor: 12,
  isClouded: true,
  isDynamicSun: false,
  isFogged: true,
  isPaused: true,
  isRainy: true,
  isThundering: true,
  isWindy: true,
  time: NOON,
};

/** The longest real step the clock takes at once, so a view shown again after a while does not skip hours. */
const LONGEST_STEP: number = 1000;

/** Metres the view moves before the modifiers are weighed again. */
const VIEW_STEP: number = 0.5;

/** Real milliseconds each transition takes. */
const TRANSITION_TIME: Readonly<Record<ERendererWeatherTransition, number>> = {
  [ERendererWeatherTransition.CUT]: 0,
  [ERendererWeatherTransition.EASE]: 250,
  [ERendererWeatherTransition.FADE]: 1500,
};

/** An effect playing, and the game seconds it has left. */
interface IPlayingEffect {
  timeline: IWeatherEffectTimeline;
  remaining: number;
}

/**
 * The weather the renderer plays by itself: its clock, the engine's pair of keyframes kept from frame to frame, the
 * effect over it, the mix seen from the view, a fade from what was shown when the consumer hands over another weather,
 * and the skies held around it all.
 */
export class WeatherPlayer {
  private readonly textures: WeatherTextures;
  private readonly pair: WeatherPair = new WeatherPair();
  private readonly thunder: WeatherThunder;
  private readonly fader: WeatherFader;
  private weather: Nullable<IRendererWeather> = null;
  private control: IRendererWeatherControl = DEFAULT_CONTROL;
  private time: number = NOON;
  private effect: Nullable<IPlayingEffect> = null;
  private view: TRendererVector = [0, 0, 0];
  private mix: Nullable<IWeatherMix> = null;
  /** What was last drawn, which a fade starts from. */
  private shown: Nullable<IRendererLighting> = null;
  private advancedAt: Nullable<number> = null;
  private isChanged: boolean = false;
  /** Whether the next change is forced, as a seek or a cut is, and shown at once. */
  private isForced: boolean = false;
  /** Whether a pair the clock never walked to was set since the last frame, as an effect starting or ending sets one. */
  private isJumped: boolean = false;
  /** Whether the last frame was lit by a strike, which the frame after it ends lights without. */
  private isFlashing: boolean = false;

  /**
   * @param textures - Where the weather's skies are put.
   * @param random - A number in `[0, 1)` each call, which bolts strike by.
   */
  public constructor(textures: RendererTextures, random: () => number = Math.random) {
    this.textures = new WeatherTextures(textures);
    this.thunder = new WeatherThunder(random);
    this.fader = new WeatherFader(this.textures);
  }

  /** Whether a weather plays, lighting the scene in place of the consumer's lighting. */
  public get isPlaying(): boolean {
    return this.weather !== null;
  }

  /** Where the weather stands, or null while none plays. */
  public get report(): Nullable<IRendererWeatherReport> {
    const { mix, effect, weather } = this;
    const pair: Nullable<TWeatherKeyframePair> = this.pair.current;

    if (!mix || !pair || !weather) {
      return null;
    }

    return {
      between: [pair[0].time, pair[1].time],
      current: this.toCurrent(weather, pair),
      effect: effect ? { name: effect.timeline.name, remaining: effect.remaining } : null,
      modifiers: mix.modifiers,
      time: this.time,
      weight: mix.weight,
    };
  }

  /**
   * @param weather - What to play from now on, or null to play nothing.
   * @param transition - How it takes over from what was shown.
   */
  public take(weather: Nullable<IRendererWeather>, transition: ERendererWeatherTransition): void {
    const duration: number = TRANSITION_TIME[transition];

    if (weather && this.shown && duration > 0) {
      this.fader.start(this.shown, duration);
    } else {
      this.fader.stop();
    }

    this.isForced = duration === 0;
    this.weather = weather;
    this.effect = null;
    this.mix = null;
    this.pair.reset();
    this.thunder.reset();
    this.isChanged = true;
    this.textures.take(weather?.textures ?? {});

    // Nothing fades into no weather: what it held goes at once.
    if (!weather) {
      this.textures.keep([]);
      this.shown = null;
    }
  }

  /**
   * @param control - How to play it from now on; a time to play from is a forced start, which ends the effect playing.
   */
  public setControl(control: IRendererWeatherControl): void {
    this.control = control;
    this.isChanged = true;

    if (control.time !== null) {
      this.time = toWeatherTimeOfDay(control.time);
      this.effect = null;
      this.pair.reset();
      this.isForced = true;
    }
  }

  /**
   * `SetWeatherFX`, or `StopWFX` for none: an effect already playing gives the cycle back first.
   *
   * @param name - The effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  public playEffect(name: Nullable<string>): void {
    const { weather } = this;

    this.stopEffect();

    const effect: Maybe<ReadonlyArray<IRendererWeatherKeyframe>> = name ? weather?.effects[name] : undefined;

    if (!name || !weather || !effect) {
      return;
    }

    this.pair.advance(weather.keyframes, this.time);

    const current: Nullable<TWeatherKeyframePair> = this.pair.current;
    const timeline: Nullable<IWeatherEffectTimeline> = current
      ? toWeatherEffectTimeline({
          current,
          cycle: weather.keyframes,
          effect,
          factor: this.control.factor,
          name,
          time: this.time,
        })
      : null;

    if (timeline) {
      this.effect = { remaining: timeline.duration, timeline };
      this.pair.set(timeline.start);
      this.isChanged = true;
      this.isJumped = true;
    }
  }

  /**
   * Moves the clock on to a frame, mixes the weather again where anything changed, and strikes over it.
   *
   * @param now - Milliseconds, as the frame loop counts them.
   * @param view - Where the camera stands, in engine space.
   * @returns What the scene is lit by now, or null where that did not change or no weather plays.
   */
  public advance(now: number, view: TRendererVector): Nullable<IRendererLighting> {
    const step: number = this.advancedAt === null ? 0 : clamp(now - this.advancedAt, 0, LONGEST_STEP);
    const { weather, control } = this;

    this.advancedAt = now;

    if (!weather) {
      return null;
    }

    if (!control.isPaused && control.factor > 0 && step > 0) {
      this.run((step / 1000) * control.factor);
    }

    if (
      weather.modifiers.length &&
      Math.hypot(...view.map((it: number, axis: number) => it - this.view[axis])) > VIEW_STEP
    ) {
      this.view = view;
      this.isChanged = true;
    }

    if (this.fader.isFading) {
      this.fader.ask(now);
      this.isChanged = true;
    }

    return this.flash(now, view, this.isChanged ? this.relight(now, weather) : null);
  }

  public dispose(): void {
    this.textures.dispose();
    this.weather = null;
    this.effect = null;
    this.mix = null;
    this.shown = null;
    this.fader.stop();
    this.thunder.reset();
  }

  /**
   * @param now - Milliseconds, as the frame loop counts them.
   * @param weather - What plays.
   * @returns What the weather shows now, faded into from what was shown.
   */
  private relight(now: number, weather: IRendererWeather): Nullable<IRendererLighting> {
    const keyframes: ReadonlyArray<IRendererWeatherKeyframe> = this.effect?.timeline.keyframes ?? weather.keyframes;

    this.isChanged = false;
    this.pair.advance(keyframes, this.time);

    const pair: Nullable<TWeatherKeyframePair> = this.pair.current;

    if (!pair) {
      return null;
    }

    this.mix = mixWeatherPair(
      { engine: weather.engine, modifiers: weather.modifiers, pair, sun: this.toSun(weather) },
      { time: this.time, view: this.view }
    );

    const target: IRendererLighting = toWeatherLighting({ control: this.control, mix: this.mix, pair, weather });

    // Faded over rather than cut to, as the skies of the engine's own blend never jump.
    if (this.isJumped && !this.isForced && !this.fader.isFading && this.shown) {
      this.fader.start(this.shown, TRANSITION_TIME[ERendererWeatherTransition.FADE]);
      this.fader.ask(now);
    }

    this.isForced = false;
    this.isJumped = false;

    const kept: Array<string> = listWeatherTextures({ keyframes, pair, weather });
    const isFading: boolean = this.fader.isFading;

    // Put before the fade asks whether they are up: a key never put reads as settled.
    this.textures.keep([...kept, ...this.fader.held]);

    const lighting: IRendererLighting = this.fader.apply(now, target);

    // Ended on this frame, what it held goes now: a paused clock may not relight again for a long while.
    if (isFading && !this.fader.isFading) {
      this.textures.keep(kept);
    }

    this.shown = lighting;

    return lighting;
  }

  /**
   * @param now - Milliseconds, as the frame loop counts them.
   * @param view - Where the camera stands, in engine space.
   * @param lighting - What the weather shows now, or null where that did not change.
   * @returns What is shown lit by the strike under way, what is shown once more the frame after one ends, or the same.
   */
  private flash(
    now: number,
    view: TRendererVector,
    lighting: Nullable<IRendererLighting>
  ): Nullable<IRendererLighting> {
    const { weather, mix, shown } = this;
    const thunder = weather?.thunder;
    const flash: Nullable<IWeatherThunderFlash> =
      thunder && mix && shown
        ? this.thunder.advance({ isEnabled: this.control.isThundering, mix, now: now / 1000, thunder, view })
        : null;

    if (flash && thunder?.settings) {
      this.isFlashing = true;

      return toThunderedLighting(lighting ?? (shown as IRendererLighting), flash, thunder.settings);
    }

    if (this.isFlashing) {
      this.isFlashing = false;

      return lighting ?? shown;
    }

    return lighting;
  }

  /** Runs the clock on, and the effect playing with it, which gives the cycle back once it has run out. */
  private run(seconds: number): void {
    this.time = toWeatherTimeOfDay(this.time + seconds);
    this.isChanged = true;

    if (this.effect) {
      this.effect.remaining -= seconds;

      if (this.effect.remaining <= 0) {
        this.stopEffect();
      }
    }
  }

  /** `StopWFX`: the cycle takes over from the effect's end, blending its keyframe there and the one after. */
  private stopEffect(): void {
    if (this.effect) {
      this.pair.set(this.effect.timeline.end);
      this.effect = null;
      this.isChanged = true;
      this.isJumped = true;
    }
  }

  private toSun(weather: IRendererWeather): TWeatherSun {
    if (weather.engine === ERendererEngine.EXTENDED && weather.sunTable) {
      return { kind: EWeatherSun.TABLE, positions: weather.sunTable };
    }

    return { kind: this.control.isDynamicSun ? EWeatherSun.DYNAMIC : EWeatherSun.AUTHORED };
  }

  /** What is mixed now as one keyframe, without the level's modifiers, which a keyframe seeded from it gets again. */
  private toCurrent(weather: IRendererWeather, pair: TWeatherKeyframePair): IRendererWeatherKeyframe {
    const mix: IWeatherMix = mixWeatherPair(
      { engine: weather.engine, modifiers: [], pair, sun: this.toSun(weather) },
      { time: this.time, view: this.view }
    );

    return toWeatherMixedKeyframe({ mix, pair, time: this.time });
  }
}
