import { Maybe, Nullable } from "@xrf/types";
import { Node, Texture, TextureNode } from "three/webgpu";

import { RendererTextures } from "#/texture/renderer-textures";

/**
 * The textures one material samples, bound through the renderer's textures and let go of with it, sampled finer by the
 * frame's texture bias where the material draws the screen.
 */
export class MaterialSamplers {
  private readonly textures: RendererTextures;
  private readonly bias: Nullable<Node<"float">>;
  private readonly bound: Array<[string, TextureNode]>;

  /**
   * @param textures - Where the textures are bound from.
   * @param bias - The mip levels every sample is moved by, or null for a material drawing no screen pixels.
   * @param bound - The bindings, shared with another view of the same material's samplers.
   */
  public constructor(
    textures: RendererTextures,
    bias: Nullable<Node<"float">> = null,
    bound: Array<[string, TextureNode]> = []
  ) {
    this.textures = textures;
    this.bias = bias;
    this.bound = bound;
  }

  /**
   * @returns The same material's samplers without the bias, for its shadow views: they draw at their own resolution.
   *   Bindings through either are let go of together.
   */
  public unbiased(): MaterialSamplers {
    return new MaterialSamplers(this.textures, null, this.bound);
  }

  /** The texture keys bound, which have to be uploaded before the material draws without a stall. */
  public get keys(): ReadonlyArray<string> {
    return this.bound.map(([key]) => key);
  }

  /**
   * @param key - The texture's key, or nothing for a slot the surface leaves empty.
   * @param placeholder - What it samples while the key holds nothing on the GPU.
   * @param coordinates - Where it samples.
   * @param isBiased - Whether the bias moves it: a sampler read at a level of its own takes none, since three prefers
   *   a bias to a level.
   * @returns The sampler.
   */
  public bind(
    key: Maybe<string>,
    placeholder: Texture,
    coordinates: Node<"vec2">,
    isBiased: boolean = true
  ): TextureNode {
    const sampler: TextureNode = this.textures.bind(key, placeholder, coordinates);

    if (key) {
      this.bound.push([key, sampler]);
    }

    // A clone that follows the bound one, so a texture swapped in once it uploads reaches it too.
    return this.bias && isBiased ? sampler.bias(this.bias) : sampler;
  }

  /** Lets every binding go, for a material that is going away. */
  public release(): void {
    this.bound.forEach(([key, sampler]) => this.textures.unbind(key, sampler));
    this.bound.length = 0;
  }
}
