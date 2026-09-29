import { Maybe } from "@xrf/types";

import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { RendererTextures } from "#/texture/renderer-textures";

/** Where one of a weather's textures is fetched from. */
type TWeatherTextureSource = IRendererWeather["textures"][string];

/** What a weather's textures are put under, apart from any key the consumer puts. */
const KEY_PREFIX: string = "@weather/";

/**
 * The textures a weather plays with, put under keys of their own: only those it draws with now are held, the rest let
 * go, such as the skies the clock no longer stands near.
 */
export class WeatherTextures {
  private readonly textures: RendererTextures;
  private sources: Readonly<Record<string, TWeatherTextureSource>> = {};
  /** The references put. */
  private readonly held: Set<string> = new Set();

  /**
   * @param textures - Where the textures are put.
   */
  public constructor(textures: RendererTextures) {
    this.textures = textures;
  }

  /**
   * @param reference - A texture reference a keyframe names.
   * @returns The key it is put under.
   */
  public static toKey(reference: string): string {
    return KEY_PREFIX + reference;
  }

  /**
   * @param sources - Where a new weather's textures are fetched from. What the last one holds stays until the next
   *   keep leaves it out, so a fade from it still has its skies.
   */
  public take(sources: Readonly<Record<string, TWeatherTextureSource>>): void {
    const kept: Record<string, TWeatherTextureSource> = {};

    for (const reference of this.held) {
      const source: Maybe<TWeatherTextureSource> = this.sources[reference];

      if (source) {
        kept[reference] = source;
      }
    }

    this.sources = { ...kept, ...sources };
  }

  /**
   * @param keys - Keys the weather's textures are put under.
   * @returns Whether every one is up, or settled on its placeholder for good.
   */
  public isUploaded(keys: Iterable<string>): boolean {
    for (const key of keys) {
      if (!this.textures.isUploaded(key)) {
        return false;
      }
    }

    return true;
  }

  /** The references held now. */
  public listHeld(): Array<string> {
    return [...this.held];
  }

  /**
   * @param references - The textures to hold now; one the weather gave no source for is left out.
   */
  public keep(references: Iterable<string>): void {
    const wanted: Set<string> = new Set();

    for (const reference of references) {
      if (reference in this.sources) {
        wanted.add(reference);
      }
    }

    for (const reference of this.held) {
      if (!wanted.has(reference)) {
        this.textures.release(WeatherTextures.toKey(reference));
        this.held.delete(reference);
      }
    }

    for (const reference of wanted) {
      const source: Maybe<TWeatherTextureSource> = this.sources[reference];

      if (source && !this.held.has(reference)) {
        this.textures.put(WeatherTextures.toKey(reference), source);
        this.held.add(reference);
      }
    }
  }

  public dispose(): void {
    this.keep([]);
    this.sources = {};
  }
}
