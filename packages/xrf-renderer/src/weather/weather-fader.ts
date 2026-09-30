import { Nullable } from "@xrf/types";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { toFadedLighting } from "#/weather/weather-fade";
import { WeatherTextures } from "#/weather/weather-textures";

/** Real milliseconds a fade waits for the skies it fades into before it starts without them. */
const LONGEST_WAIT: number = 2000;

/** A fade from what was shown into what the weather shows now. */
interface IWeatherFading {
  from: IRendererLighting;
  /** Milliseconds it was asked for at, or null until the frame after. */
  askedAt: Nullable<number>;
  /** Milliseconds it started at, or null while the skies it fades into are still going up. */
  startedAt: Nullable<number>;
  duration: number;
  /** The textures what it fades from shows, held until it ends. */
  held: ReadonlyArray<string>;
}

/**
 * A fade from what was shown into what the weather shows now: it waits for the skies it fades into to go up, two
 * seconds at most, then runs its time, holding what it fades from shows until it ends.
 */
export class WeatherFader {
  private readonly textures: WeatherTextures;
  private fading: Nullable<IWeatherFading> = null;

  /**
   * @param textures - The weather's textures, which a fade holds and waits on.
   */
  public constructor(textures: WeatherTextures) {
    this.textures = textures;
  }

  /** Whether a fade waits or runs. */
  public get isFading(): boolean {
    return this.fading !== null;
  }

  /** The textures what the fade fades from shows, held until it ends. */
  public get held(): ReadonlyArray<string> {
    return this.fading?.held ?? [];
  }

  /**
   * Starts a fade, in place of any before it, which waits from the frame that first asks for it.
   *
   * @param from - What is shown.
   * @param duration - Real milliseconds it takes once its skies are up.
   */
  public start(from: IRendererLighting, duration: number): void {
    this.fading = { askedAt: null, duration, from, held: this.textures.listHeld(), startedAt: null };
  }

  public stop(): void {
    this.fading = null;
  }

  /**
   * @param now - Milliseconds a frame asks for the fade at, which it waits for its skies from the first time.
   */
  public ask(now: number): void {
    if (this.fading) {
      this.fading.askedAt ??= now;
    }
  }

  /**
   * @param now - Milliseconds, as the frame loop counts them.
   * @param target - What the weather shows now.
   * @returns What to draw: the fade under way, or the weather where none is.
   */
  public apply(now: number, target: IRendererLighting): IRendererLighting {
    const { fading } = this;

    if (!fading) {
      return target;
    }

    if (
      fading.startedAt === null &&
      (this.textures.isUploaded(listSkyKeys(target)) || now - (fading.askedAt ?? now) >= LONGEST_WAIT)
    ) {
      fading.startedAt = now;
    }

    const progress: number = fading.startedAt === null ? 0 : (now - fading.startedAt) / fading.duration;

    if (progress >= 1) {
      this.fading = null;
    }

    return toFadedLighting({ from: fading.from, progress, to: target });
  }
}

/** Every key a lighting's sky draws. */
function listSkyKeys(lighting: IRendererLighting): Array<string> {
  const { sky } = lighting;

  return [...sky.textures, ...sky.environments, ...(sky.clouds?.textures ?? [])].filter(
    (key: Nullable<string>): key is string => key !== null
  );
}
