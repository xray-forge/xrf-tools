import { Nullable } from "@xrf/types";
import { CubeTextureNode } from "three/webgpu";

import { IRendererSky } from "#/contract/renderer-sky";
import { getPlaceholderSkyTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/**
 * The two skies the lighting names and their irradiance cubes, bound into what samples them by their texture keys:
 * each draws its placeholder until the consumer's cube is up, then the cube.
 */
export class SceneSky {
  private readonly textures: RendererTextures;
  private readonly sky: SkyUniforms;
  /** The two skies' keys, then their irradiance cubes'. */
  private readonly keys: Array<Nullable<string>> = [null, null, null, null];
  private readonly samplers: ReadonlyArray<CubeTextureNode>;

  /**
   * @param textures - Where the skies' cubes are put.
   * @param sky - What samples them.
   */
  public constructor(textures: RendererTextures, sky: SkyUniforms) {
    this.textures = textures;
    this.sky = sky;
    this.samplers = [...sky.cubes, ...sky.environments];
  }

  /**
   * @param sky - The skies the lighting names now.
   */
  public take(sky: IRendererSky): void {
    [...sky.textures, ...sky.environments].forEach((key: string, index: number) => {
      const previous: Nullable<string> = this.keys[index];

      if (previous === key) {
        return;
      }

      if (previous) {
        this.textures.unbind(previous, this.samplers[index]);
      }

      this.textures.target(key, getPlaceholderSkyTexture(), this.samplers[index]);
      this.keys[index] = key;
    });
  }

  /** Lights the hemisphere by the irradiance cubes once both are up, by the lighting's stand-in until then. */
  public update(): void {
    const [, , first, second] = this.keys;
    const isUp: boolean =
      first !== null &&
      second !== null &&
      this.textures.getUploaded(first) !== null &&
      this.textures.getUploaded(second) !== null;

    this.sky.environmentsUp.value = isUp ? 1 : 0;
  }

  public dispose(): void {
    this.keys.forEach((key: Nullable<string>, index: number) => {
      if (key) {
        this.textures.unbind(key, this.samplers[index]);
      }
    });
    this.keys.fill(null);
  }
}
