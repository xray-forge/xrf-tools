import { Maybe, Nullable } from "@xrf/types";
import { Data3DTexture, Texture } from "three/webgpu";

import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IDdsVolume } from "#/dds/dds-volume";
import { readDdsVolume } from "#/dds/dds-volume-read";
import { isSameDefinition } from "#/scene/same-definition";
import { fetchRendererBytes } from "#/texture/fetch-renderer-bytes";
import { getNeutralDetailTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { prepareVolume, WetUniforms } from "#/uniforms/wet-uniforms";
import { WeatherTextures } from "#/weather/weather-textures";

/** The textures a weather wets surfaces with: the flow's reference, and where the splashes' volume is fetched from. */
interface IWetTaken {
  flow: string;
  splash: Nullable<TRendererTextureSource>;
}

/**
 * The textures rain wets surfaces with, for the weather that names them: the flow through the texture store, and the
 * splashes' volume fetched and decoded here, since the store refuses a volume and a GPU samples a compressed one only
 * with a feature it may lack.
 */
export class SceneWet {
  private readonly textures: RendererTextures;
  private readonly wet: WetUniforms;
  private flowKey: Nullable<string> = null;
  /** What was last taken, which a weather naming the same textures keeps. */
  private taken: Nullable<IWetTaken> = null;
  private volume: Nullable<Data3DTexture> = null;
  private fetching: Nullable<AbortController> = null;
  private readonly neutral: Texture;

  /**
   * @param textures - Where the flow is put.
   * @param wet - What the wet surfaces' shaders read.
   */
  public constructor(textures: RendererTextures, wet: WetUniforms) {
    this.textures = textures;
    this.wet = wet;
    this.neutral = wet.splash.value;
  }

  /**
   * @param weather - What plays from now on, or null for nothing.
   */
  public take(weather: Nullable<IRendererWeather>): void {
    const surfaces = weather?.wet;
    const source: Maybe<TRendererTextureSource> = surfaces ? weather?.textures[surfaces.splash] : undefined;
    const taken: Nullable<IWetTaken> = surfaces ? { flow: surfaces.flow, splash: source ?? null } : null;

    // A keyframe edited by hand sends its weather again, naming the same textures: nothing is fetched again for it.
    if (isSameDefinition(taken, this.taken)) {
      return;
    }

    this.release();
    this.taken = taken;

    if (!taken) {
      return;
    }

    this.flowKey = WeatherTextures.toKey(taken.flow);
    this.textures.target(this.flowKey, getNeutralDetailTexture(), this.wet.flow);

    if (taken.splash?.encoding === ERendererTextureEncoding.FETCH) {
      const fetching: AbortController = new AbortController();

      this.fetching = fetching;
      void this.load(taken.splash.file, fetching);
    }
  }

  public dispose(): void {
    this.release();
  }

  private async load(
    file: Extract<TRendererTextureSource, { encoding: ERendererTextureEncoding.FETCH }>["file"],
    fetching: AbortController
  ): Promise<void> {
    try {
      const volume: Nullable<IDdsVolume> = readDdsVolume((await fetchRendererBytes(file, fetching.signal)).bytes);

      if (fetching.signal.aborted) {
        return;
      }

      if (!volume) {
        console.error("The rain's splash volume is no DXT volume, so wet surfaces do not ripple.");

        return;
      }

      const texture: Data3DTexture = new Data3DTexture(volume.rgba, volume.width, volume.height, volume.depth);

      prepareVolume(texture);
      this.volume = texture;
      this.wet.splash.value = texture;
    } catch (error: unknown) {
      if (!fetching.signal.aborted) {
        console.error("The rain's splash volume failed to load, so wet surfaces do not ripple:", error);
      }
    }
  }

  private release(): void {
    this.fetching?.abort();
    this.fetching = null;
    this.taken = null;

    if (this.flowKey) {
      this.textures.unbind(this.flowKey, this.wet.flow);
      this.flowKey = null;
    }

    this.wet.splash.value = this.neutral;
    this.volume?.dispose();
    this.volume = null;
  }
}
