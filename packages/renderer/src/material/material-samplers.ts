import { Maybe } from "@xrf/types";
import { Node, Texture, TextureNode } from "three/webgpu";

import { RendererTextures } from "#/texture/renderer-textures";

/**
 * The textures one material samples, bound through the renderer's textures and let go of with it.
 */
export class MaterialSamplers {
  private readonly textures: RendererTextures;
  private readonly bound: Array<[string, TextureNode]> = [];

  public constructor(textures: RendererTextures) {
    this.textures = textures;
  }

  /** The texture keys bound, which have to be uploaded before the material draws without a stall. */
  public get keys(): ReadonlyArray<string> {
    return this.bound.map(([key]) => key);
  }

  /**
   * @param key - The texture's key, or nothing for a slot the surface leaves empty.
   * @param placeholder - What it samples while the key holds nothing on the GPU.
   * @param coordinates - Where it samples.
   * @returns The sampler.
   */
  public bind(key: Maybe<string>, placeholder: Texture, coordinates: Node<"vec2">): TextureNode {
    const sampler: TextureNode = this.textures.bind(key, placeholder, coordinates);

    if (key) {
      this.bound.push([key, sampler]);
    }

    return sampler;
  }

  /** Lets every binding go, for a material that is going away. */
  public release(): void {
    this.bound.forEach(([key, sampler]) => this.textures.unbind(key, sampler));
    this.bound.length = 0;
  }
}
