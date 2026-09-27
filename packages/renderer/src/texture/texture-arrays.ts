import { Maybe, Nullable } from "@xrf/types";
import { CompressedTexture, Texture, WebGPURenderer } from "three/webgpu";

import { TextureArray } from "#/texture/texture-array";
import { toTextureArrayClass } from "#/texture/texture-array-class";

/** WebGPU's default `maxTextureArrayLayers`, which a device three opens without asking for more has. */
export const DEFAULT_ARRAY_LAYER_LIMIT: number = 256;

/** Where a key's texture is held: its array and its layer. */
export interface ITextureLayer {
  array: TextureArray;
  layer: number;
}

/** A key's claims: where it is held, the class it was held as, and by how many surfaces. */
interface ITextureClaim {
  held: ITextureLayer;
  kind: string;
  users: number;
}

/** Every texture array, by class: a key joins the first of its class with room, and is held once however claimed. */
export class TextureArrays {
  /** Layers the device allows an array, which the device says once it is open. */
  public layerLimit: number = DEFAULT_ARRAY_LAYER_LIMIT;

  private readonly arrays: Map<string, Array<TextureArray>> = new Map();
  private readonly claims: Map<string, ITextureClaim> = new Map();
  private readonly onReplaced: (key: string) => void;
  private made: number = 0;

  /**
   * @param onReplaced - Told an array's key where the array was replaced, which bundles binding it record again for.
   */
  public constructor(onReplaced: (key: string) => void) {
    this.onReplaced = onReplaced;
  }

  /**
   * @param key - A texture's key, sampled from an array by one more surface.
   * @param texture - What the key holds on the GPU.
   * @returns Where it is held, or null for a texture no array holds.
   */
  public claim(key: string, texture: Texture): Nullable<ITextureLayer> {
    const kind: Nullable<string> = toTextureArrayClass(texture);
    const claim: Maybe<ITextureClaim> = this.claims.get(key);

    if (!kind) {
      return null;
    }

    if (claim && claim.kind === kind) {
      claim.users += 1;
      claim.held.array.claim(key, texture);

      return claim.held;
    }

    // Held as another class before: every surface sampling it moves when it is claimed again.
    if (claim) {
      return null;
    }

    const array: TextureArray = this.findRoom(kind, texture as CompressedTexture);
    const layer: number = array.claim(key, texture) as number;
    const held: ITextureLayer = { array, layer };

    this.claims.set(key, { held, kind, users: 1 });

    return held;
  }

  /**
   * @param key - A texture's key one surface no longer samples from its array.
   */
  public release(key: string): void {
    const claim: Maybe<ITextureClaim> = this.claims.get(key);

    if (!claim) {
      return;
    }

    claim.held.array.release(key);

    if (--claim.users <= 0) {
      this.claims.delete(key);
    }
  }

  /**
   * @param key - A texture's key whose texture was replaced.
   * @param texture - What it holds now, or nothing.
   * @returns Whether the key is held as it was: false where it is not held, holds nothing now, or holds another class.
   */
  public refresh(key: string, texture: Nullable<Texture>): boolean {
    const claim: Maybe<ITextureClaim> = this.claims.get(key);

    if (!claim || !texture || toTextureArrayClass(texture) !== claim.kind) {
      return false;
    }

    claim.held.array.refresh(key, texture);

    return true;
  }

  /**
   * Copies what waits into each array, growing any that had to.
   *
   * @param renderer - The renderer drawing.
   */
  public flush(renderer: WebGPURenderer): void {
    for (const arrays of this.arrays.values()) {
      for (const array of arrays) {
        if (array.flush(renderer)) {
          this.onReplaced(array.key);
        }
      }
    }
  }

  public dispose(): void {
    this.arrays.forEach((arrays: Array<TextureArray>) => arrays.forEach((array: TextureArray) => array.dispose()));
    this.arrays.clear();
    this.claims.clear();
  }

  /** The first array of a class with room, or a new one. */
  private findRoom(kind: string, prototype: CompressedTexture): TextureArray {
    let arrays: Maybe<Array<TextureArray>> = this.arrays.get(kind);

    if (!arrays) {
      arrays = [];
      this.arrays.set(kind, arrays);
    }

    let array: Maybe<TextureArray> = arrays.find((it: TextureArray) => it.hasRoom);

    if (!array) {
      array = new TextureArray(`texture-array:${this.made++}`, prototype, this.layerLimit);
      arrays.push(array);
    }

    return array;
  }
}
