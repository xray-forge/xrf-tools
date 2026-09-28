import { Maybe, Nullable } from "@xrf/types";
import { CompressedTexture, Texture, WebGPURenderer } from "three/webgpu";

import { ITextureCopy } from "#/internals/texture-copy";
import { TextureArray } from "#/texture/texture-array";
import { toTextureArrayClass } from "#/texture/texture-array-class";
import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { ITextureLayer } from "#/texture/texture-layer";

/** WebGPU's default `maxTextureArrayLayers`, which a device three opens without asking for more has. */
export const DEFAULT_ARRAY_LAYER_LIMIT: number = 256;

/** Where a key is held, and the class it was held as. */
interface ITextureClaim {
  held: ITextureLayer;
  kind: string;
}

/**
 * Every texture array, by class: a key joins the first of its class with room and is held once however many surfaces
 * claim it; an array left holding nothing goes with the next flush.
 */
export class TextureArrays {
  /** Layers the device allows an array, which the device says once it is open. */
  public layerLimit: number = DEFAULT_ARRAY_LAYER_LIMIT;

  private readonly arrays: Map<string, Array<TextureArray>> = new Map();
  private readonly claims: Map<string, ITextureClaim> = new Map();
  /**
   * A zeroed buffer of each size an array asked for since the last `releaseZeros`, which every array of that size made
   * meanwhile writes its first layer from.
   */
  private readonly zeros: Map<number, Uint8Array> = new Map();
  private readonly onReplaced: (key: string) => void;
  private made: number = 0;

  /**
   * @param onReplaced - Told an array's key where the array was replaced or went, which bundles binding it record
   *   again for.
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
      claim.held.array.claim(key, texture);

      return claim.held;
    }

    // Held as another class: sampled as its own until it is detached, which a texture changing class is.
    if (claim) {
      return null;
    }

    const array: TextureArray = this.findRoom(kind, texture as CompressedTexture);
    const layer: number = array.claim(key, texture) as number;
    const held: ITextureLayer = { array, layer };

    this.claims.set(key, { held, kind });

    return held;
  }

  /**
   * @param key - A texture's key.
   * @returns Whether an array holds it.
   */
  public holds(key: string): boolean {
    return this.claims.has(key);
  }

  /**
   * @param key - A texture's key, sampled from its array by one more surface, whatever its own texture holds now.
   * @returns Where it is held, or null for a key no array holds.
   */
  public retain(key: string): Nullable<ITextureLayer> {
    const claim: Maybe<ITextureClaim> = this.claims.get(key);

    return claim && claim.held.array.retain(key) ? claim.held : null;
  }

  /**
   * @param key - A texture's key one surface no longer samples from an array.
   * @param held - Where that surface sampled it: where it was claimed, whether or not the key was detached since.
   */
  public release(key: string, held: ITextureLayer): void {
    if (held.array.release(key) && this.claims.get(key)?.held.array === held.array) {
      this.claims.delete(key);
    }
  }

  /**
   * Lets the surfaces claiming a key from now on hold it anew, as the class its texture is now: the ones holding it
   * still keep its layer as it was until they let it go.
   *
   * @param key - A texture's key whose texture changed class, or holds nothing now.
   */
  public detach(key: string): void {
    const claim: Maybe<ITextureClaim> = this.claims.get(key);

    if (claim) {
      claim.held.array.detach(key);
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
   * @param key - A texture's key whose own texture was let go.
   * @param texture - That texture, to be filled again on the GPU.
   * @returns The copies filling it from its layer, or null where no layer holds a copy of it.
   */
  public toRestoreCopies(key: string, texture: Texture): Nullable<Array<ITextureCopy>> {
    return this.claims.get(key)?.held.array.toRestoreCopies(key, texture) ?? null;
  }

  /**
   * @param skipped - Copies the frame could not make, which are rare: a scan of the claims each.
   * @returns The keys they were for, each copied again with the next flush where it is still claimed there.
   */
  public retry(skipped: ReadonlyArray<ITextureCopy>): Set<string> {
    const keys: Set<string> = new Set();

    for (const copy of skipped) {
      for (const [key, { held }] of this.claims) {
        if (held.array.retry(key, copy)) {
          keys.add(key);
        }
      }
    }

    return keys;
  }

  /**
   * Says what every array waits for, telling of each array replaced by a larger one, and lets every array holding
   * nothing go.
   *
   * @returns The copies, in order, and what to dispose once they are sent.
   */
  public flush(): ITextureArrayFlush {
    const flush: ITextureArrayFlush = { copies: [], disposals: [], evicted: [], replaced: [] };

    for (const [kind, arrays] of this.arrays) {
      for (const array of arrays) {
        const { copies, disposals, evicted, replaced } = array.flush();

        flush.copies.push(...copies);
        flush.disposals.push(...disposals);
        flush.evicted.push(...evicted);
        flush.replaced.push(...replaced);
      }

      // Emptied: what still binds it records again, and it goes once this flush's copies are sent.
      const kept: Array<TextureArray> = arrays.filter((array: TextureArray) => {
        if (!array.isEmpty) {
          return true;
        }

        flush.disposals.push(...array.listDisposals());
        flush.replaced.push(array.key);

        return false;
      });

      if (kept.length) {
        this.arrays.set(kind, kept);
      } else {
        this.arrays.delete(kind);
      }
    }

    flush.replaced.forEach((key: string) => this.onReplaced(key));

    return flush;
  }

  /**
   * Fits every array gone unchanged a while to what it holds.
   *
   * @param now - Milliseconds, on `performance.now()`'s clock.
   */
  public compact(now: number): void {
    this.arrays.forEach((arrays: Array<TextureArray>) => arrays.forEach((array: TextureArray) => array.compact(now)));
  }

  /**
   * Lets go of the zeroed layer of every array on the GPU now, and of the zeroed buffers made for them: an array made
   * later makes its own.
   *
   * @param renderer - The renderer that uploaded them.
   */
  public releaseZeros(renderer: WebGPURenderer): void {
    this.arrays.forEach((arrays: Array<TextureArray>) =>
      arrays.forEach((array: TextureArray) => array.releaseZeros(renderer))
    );
    this.zeros.clear();
  }

  public dispose(): void {
    this.arrays.forEach((arrays: Array<TextureArray>) => arrays.forEach((array: TextureArray) => array.dispose()));
    this.arrays.clear();
    this.claims.clear();
    this.zeros.clear();
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
      array = new TextureArray(`texture-array:${this.made++}`, prototype, this.layerLimit, (bytes: number) =>
        this.toZeros(bytes)
      );
      arrays.push(array);
    }

    return array;
  }

  private toZeros(bytes: number): Uint8Array {
    let zeros: Maybe<Uint8Array> = this.zeros.get(bytes);

    if (!zeros) {
      zeros = new Uint8Array(bytes);
      this.zeros.set(bytes, zeros);
    }

    return zeros;
  }
}
