import { IRendererWeather, IRendererWeatherKeyframe } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeather, toLevelManualKeyframe } from "@/core/level/lib/weather/level-manual-weather";
import { ILevelManualWeatherBuilderInput } from "@/core/level/lib/weather/level-manual-weather-builder-input";
import { toLevelRendererEngine, toLevelRendererTextures } from "@/core/level/lib/weather/level-renderer-weather";
import { TLevelRendererWeatherBase } from "@/core/level/lib/weather/level-renderer-weather-base";

/** A part of the weather built for some key, handed over again while the key stays. */
interface IBuilt<T> {
  key: string;
  value: T;
}

/**
 * Builds what the renderer plays of the keyframe set by hand, as a cycle of one over the level's weather. Each part an
 * edit leaves as it was is handed over as the same object, so the renderer is sent the parts that changed alone.
 *
 * It stands its sun by its own angles on either engine, so no sun table goes with it, and an effect played over it
 * that stands no sun of its own stands it where the keyframe does.
 */
export class LevelManualWeatherBuilder {
  private readonly base: TLevelRendererWeatherBase;
  private readonly roots: ILevelManualWeatherBuilderInput["roots"];
  private effects: Nullable<IBuilt<IRendererWeather["effects"]>> = null;
  private textures: Nullable<IBuilt<IRendererWeather["textures"]>> = null;

  public constructor(input: ILevelManualWeatherBuilderInput) {
    this.roots = input.roots;
    this.base = input.base ?? {
      effects: {},
      engine: toLevelRendererEngine(input.engine),
      modifiers: [],
      rain: null,
      sunTable: null,
      textures: {},
      thunder: null,
      wet: null,
    };
  }

  /**
   * @param manual - The keyframe set by hand.
   * @param located - What its own textures resolved to.
   * @returns What the renderer plays.
   */
  public async build(
    manual: ILevelManualWeather,
    located: ReadonlyArray<LevelTextureReference>
  ): Promise<IRendererWeather> {
    const keyframe: IRendererWeatherKeyframe = toLevelManualKeyframe(manual, 0);

    return {
      ...this.base,
      effects: this.toEffects(keyframe),
      keyframes: [keyframe],
      sunTable: null,
      textures: await this.toTextures(located),
    };
  }

  /** The level's effects, each keyframe that stands no sun standing it where the keyframe set by hand does. */
  private toEffects(keyframe: IRendererWeatherKeyframe): IRendererWeather["effects"] {
    const key: string = JSON.stringify(keyframe.sunDirection);

    if (this.effects?.key !== key) {
      this.effects = {
        key,
        value: Object.fromEntries(
          Object.entries(this.base.effects).map(([name, keyframes]) => [
            name,
            keyframes.map((it: IRendererWeatherKeyframe) =>
              it.sunDirection ? it : { ...it, sunDirection: keyframe.sunDirection }
            ),
          ])
        ),
      };
    }

    return this.effects.value;
  }

  /** Where the level's textures and the keyframe's own are fetched from. */
  private async toTextures(located: ReadonlyArray<LevelTextureReference>): Promise<IRendererWeather["textures"]> {
    const key: string = JSON.stringify(located);

    if (this.textures?.key !== key) {
      this.textures = {
        key,
        value: { ...this.base.textures, ...(await toLevelRendererTextures(this.roots, located)) },
      };
    }

    return this.textures.value;
  }
}
