import { Nullable } from "@xrf/types";

import { IRendererSky } from "#/contract/renderer-lighting";
import { getPlaceholderSkyTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

/**
 * The two skies the lighting names, bound into what samples them by their texture keys: each draws its placeholder
 * until the consumer's cube is up, then the cube.
 */
export class SceneSky {
  private readonly textures: RendererTextures;
  private readonly sky: SkyUniforms;
  private readonly keys: [Nullable<string>, Nullable<string>] = [null, null];

  /**
   * @param textures - Where the skies' cubes are put.
   * @param sky - What samples them.
   */
  public constructor(textures: RendererTextures, sky: SkyUniforms) {
    this.textures = textures;
    this.sky = sky;
  }

  /**
   * @param sky - The skies the lighting names now.
   */
  public take(sky: IRendererSky): void {
    sky.textures.forEach((key: string, index: number) => {
      const previous: Nullable<string> = this.keys[index];

      if (previous === key) {
        return;
      }

      if (previous) {
        this.textures.unbind(previous, this.sky.cubes[index]);
      }

      this.textures.target(key, getPlaceholderSkyTexture(), this.sky.cubes[index]);
      this.keys[index] = key;
    });
  }

  public dispose(): void {
    this.keys.forEach((key: Nullable<string>, index: number) => {
      if (key) {
        this.textures.unbind(key, this.sky.cubes[index]);
      }
    });
    this.keys.fill(null);
  }
}
