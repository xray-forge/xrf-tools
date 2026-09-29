import { Maybe } from "@xrf/types";

import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { RendererTextures } from "#/texture/renderer-textures";

/** Where one sky of a weather is fetched from. */
type TWeatherTextureSource = IRendererWeather["textures"][string];

/** What a weather's textures are put under, apart from any key the consumer puts. */
const KEY_PREFIX: string = "@weather/";

/**
 * The skies a weather plays, put under keys of their own: only those the clock stands near are held, the rest let go.
 */
export class WeatherTextures {
  private readonly textures: RendererTextures;
  private sources: Readonly<Record<string, TWeatherTextureSource>> = {};
  /** The references put. */
  private readonly held: Set<string> = new Set();

  /**
   * @param textures - Where the skies are put.
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
   * @param sources - Where a new weather's skies are fetched from. What the last one holds stays until the next keep
   *   leaves it out, so a fade from it still has its skies.
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

  /** The references held now. */
  public listHeld(): Array<string> {
    return [...this.held];
  }

  /**
   * @param references - The skies to hold now; one the weather gave no source for is left out.
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
