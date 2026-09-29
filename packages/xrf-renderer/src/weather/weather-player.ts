import { toDegrees } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IRendererWeatherReport } from "#/contract/weather/renderer-weather-report";
import { DEFAULT_RENDERER_GRASS_WIND, DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { RendererTextures } from "#/texture/renderer-textures";
import { toWeatherTimeOfDay } from "#/weather/weather-day";
import { IWeatherEffectTimeline, toWeatherEffectTimeline } from "#/weather/weather-effect-timeline";
import { IWeatherMix } from "#/weather/weather-mix";
import { mixWeather } from "#/weather/weather-mixer";
import { EWeatherSun, TWeatherSun } from "#/weather/weather-sun";
import { WeatherTextures } from "#/weather/weather-textures";

/** Seconds from midnight to noon. */
const NOON: number = 12 * 60 * 60;

/** How a weather plays until told otherwise: paused at noon, at the engine's own time factor. */
const DEFAULT_CONTROL: IRendererWeatherControl = {
  factor: 12,
  isDynamicSun: false,
  isFogged: true,
  isPaused: true,
  isWindy: true,
  time: NOON,
};

/** The longest real step the clock takes at once, so a view shown again after a while does not skip hours. */
const LONGEST_STEP: number = 1000;

/** `EPS_L`, under which it does not rain at all. */
const RAIN_THRESHOLD: number = 0.001;

/** Metres the view moves before the modifiers are weighed again. */
const VIEW_STEP: number = 0.5;

/** An effect playing, and the game seconds it has left. */
interface IPlayingEffect {
  timeline: IWeatherEffectTimeline;
  remaining: number;
}

/**
 * The weather the renderer plays by itself: its clock, the effect over it, the mix at the clock's time seen from the
 * view, and the skies held around it.
 */
export class WeatherPlayer {
  private readonly textures: WeatherTextures;
  private weather: Nullable<IRendererWeather> = null;
  private control: IRendererWeatherControl = DEFAULT_CONTROL;
  private time: number = NOON;
  private effect: Nullable<IPlayingEffect> = null;
  private view: TRendererVector = [0, 0, 0];
  private mix: Nullable<IWeatherMix> = null;
  private advancedAt: Nullable<number> = null;
  private isChanged: boolean = false;

  /**
   * @param textures - Where the weather's skies are put.
   */
  public constructor(textures: RendererTextures) {
    this.textures = new WeatherTextures(textures);
  }

  /** Whether a weather plays, lighting the scene in place of the consumer's lighting. */
  public get isPlaying(): boolean {
    return this.weather !== null;
  }

  /** Where the weather stands, or null while none plays. */
  public get report(): Nullable<IRendererWeatherReport> {
    const { mix, effect } = this;

    return mix
      ? {
          effect: effect ? { name: effect.timeline.name, remaining: effect.remaining } : null,
          keyframes: effect ? null : mix.keyframes,
          modifiers: mix.modifiers,
          time: this.time,
          weight: mix.weight,
        }
      : null;
  }

  /**
   * @param weather - What to play from now on, or null to play nothing.
   */
  public take(weather: Nullable<IRendererWeather>): void {
    this.weather = weather;
    this.effect = null;
    this.mix = null;
    this.isChanged = true;
    this.textures.take(weather?.textures ?? {});
  }

  /**
   * @param control - How to play it from now on; a time to play from ends the effect playing.
   */
  public setControl(control: IRendererWeatherControl): void {
    this.control = control;
    this.isChanged = true;

    if (control.time !== null) {
      this.time = toWeatherTimeOfDay(control.time);
      this.effect = null;
    }
  }

  /**
   * @param name - The effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  public playEffect(name: Nullable<string>): void {
    const { weather } = this;
    const effect: Maybe<ReadonlyArray<IRendererWeatherKeyframe>> = name ? weather?.effects[name] : undefined;
    const timeline: Nullable<IWeatherEffectTimeline> =
      name && weather && effect
        ? toWeatherEffectTimeline({
            cycle: weather.keyframes,
            effect,
            factor: this.control.factor,
            name,
            time: this.time,
          })
        : null;

    this.effect = timeline ? { remaining: timeline.duration, timeline } : null;
    this.isChanged = true;
  }

  /**
   * Moves the clock on to a frame, and mixes the weather again where anything changed.
   *
   * @param now - Milliseconds, as the frame loop counts them.
   * @param view - Where the camera stands, in engine space.
   * @returns What the scene is lit by now, or null where that did not change or no weather plays.
   */
  public advance(now: number, view: TRendererVector): Nullable<IRendererLighting> {
    const step: number = this.advancedAt === null ? 0 : Math.min(Math.max(now - this.advancedAt, 0), LONGEST_STEP);
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

    if (!this.isChanged) {
      return null;
    }

    const keyframes: ReadonlyArray<IRendererWeatherKeyframe> = this.effect?.timeline.keyframes ?? weather.keyframes;

    this.isChanged = false;
    this.mix = mixWeather(
      { engine: weather.engine, keyframes, modifiers: weather.modifiers, sun: this.toSun(weather) },
      { time: this.time, view: this.view }
    );

    if (!this.mix) {
      return null;
    }

    this.textures.keep([...WeatherPlayer.listNear(keyframes, this.mix), ...WeatherPlayer.listRain(weather)]);

    return this.toLighting(keyframes, this.mix);
  }

  public dispose(): void {
    this.textures.dispose();
    this.weather = null;
    this.effect = null;
    this.mix = null;
  }

  /** Runs the clock on, and the effect playing with it, which gives the cycle back once it has run out. */
  private run(seconds: number): void {
    this.time = toWeatherTimeOfDay(this.time + seconds);
    this.isChanged = true;

    if (this.effect) {
      this.effect.remaining -= seconds;

      if (this.effect.remaining <= 0) {
        this.effect = null;
      }
    }
  }

  private toSun(weather: IRendererWeather): TWeatherSun {
    if (weather.engine === ERendererWeatherEngine.EXTENDED && weather.sunTable) {
      return { kind: EWeatherSun.TABLE, positions: weather.sunTable };
    }

    return { kind: this.control.isDynamicSun ? EWeatherSun.DYNAMIC : EWeatherSun.AUTHORED };
  }

  private toLighting(keyframes: ReadonlyArray<IRendererWeatherKeyframe>, mix: IWeatherMix): IRendererLighting {
    const { isFogged, isWindy } = this.control;
    const a: IRendererWeatherKeyframe = keyframes[mix.keyframes[0]];
    const b: IRendererWeatherKeyframe = keyframes[mix.keyframes[1]];
    const [x, y, z]: TRendererVector = mix.sunDirection;

    return {
      ambientColor: mix.ambientColor,
      fog: isFogged
        ? { color: mix.fogColor, density: mix.fogDensity, distance: mix.fogDistance, farPlane: mix.farPlane }
        : null,
      grass: isWindy ? DEFAULT_RENDERER_GRASS_WIND : null,
      hemisphereColor: [mix.hemiColor[0], mix.hemiColor[1], mix.hemiColor[2]],
      sky: {
        blend: mix.weight,
        clouds: {
          color: mix.cloudsColor,
          rotation: toDegrees(mix.cloudsRotation),
          textures: [WeatherPlayer.toCloudsKey(a), WeatherPlayer.toCloudsKey(b)],
        },
        color: mix.skyColor,
        environments: [WeatherTextures.toKey(a.skyTextureEnv), WeatherTextures.toKey(b.skyTextureEnv)],
        rotation: toDegrees(mix.skyRotation),
        textures: [WeatherTextures.toKey(a.skyTexture), WeatherTextures.toKey(b.skyTexture)],
      },
      skyIrradiance: DEFAULT_RENDERER_LIGHTING.skyIrradiance,
      sunColor: mix.sunColor,
      // Engine `z` negated into renderer space.
      sunDirection: [x, y, -z],
      trees: isWindy
        ? { amplitude: mix.treeAmplitude, rotation: mix.treeRotation, speed: mix.treeSpeed, wave: mix.treeWave }
        : null,
      waterIntensity: mix.waterIntensity,
      rain:
        this.weather?.rain && mix.rainDensity >= RAIN_THRESHOLD
          ? {
              color: mix.rainColor,
              density: mix.rainDensity,
              windDirection: mix.windDirection,
              windVelocity: mix.windVelocity,
            }
          : null,
    };
  }

  /** The key a keyframe's clouds are put under, or null for a keyframe that names none. */
  private static toCloudsKey(keyframe: IRendererWeatherKeyframe): Nullable<string> {
    return keyframe.cloudsTexture ? WeatherTextures.toKey(keyframe.cloudsTexture) : null;
  }

  /** The rain's textures, held while its weather plays so a shower starting has them. */
  private static listRain(weather: IRendererWeather): Array<string> {
    const { rain } = weather;

    return rain ? [rain.streak, ...(rain.drop ? [rain.drop.texture] : [])] : [];
  }

  /** The skies of the two keyframes mixed and of the one after, fetched before the clock reaches it. */
  private static listNear(keyframes: ReadonlyArray<IRendererWeatherKeyframe>, mix: IWeatherMix): Array<string> {
    const next: Maybe<IRendererWeatherKeyframe> = keyframes[(mix.keyframes[1] + 1) % keyframes.length];

    return [keyframes[mix.keyframes[0]], keyframes[mix.keyframes[1]], next].flatMap(
      (keyframe: Maybe<IRendererWeatherKeyframe>) =>
        keyframe ? [keyframe.skyTexture, keyframe.skyTextureEnv, keyframe.cloudsTexture] : []
    );
  }
}
