import { Maybe, Nullable } from "@xrf/types";
import {
  CompressedArrayTexture,
  CompressedPixelFormat,
  CompressedTexture,
  RED_RGTC1_Format,
  RGBA_S3TC_DXT1_Format,
  SIGNED_RED_RGTC1_Format,
  Texture,
} from "three/webgpu";

import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { ITextureTarget } from "#/texture/texture-target";

/** Layers an array holds before it first grows. */
const INITIAL_LAYERS: number = 4;

/** How much an array grows by, which leaves at most a third of a grown one unused. */
const GROWTH: number = 1.5;

/** Milliseconds an array has to go unchanged before it is fitted: a level streaming in claims a layer at a time. */
const COMPACT_IDLE: number = 3000;

/** Texels a block covers, a side. */
const BLOCK_SIDE: number = 4;

/** One key's layer, and how many surfaces sample it there. */
interface ITextureArrayLayer {
  layer: number;
  users: number;
  /** What the layer was last copied from, or is still to be. */
  source: Texture;
}

/** A level's physical texels a side, whole blocks, and its bytes a layer. */
interface ILevelExtent {
  width: number;
  height: number;
  bytes: number;
}

/**
 * Textures of one class as the layers of one array, each copied on the GPU from its key's own texture, which then goes;
 * grown by half again to the device's limit, and fitted to what it holds once asked.
 */
export class TextureArray {
  /** What the textures' users are told by when the array is replaced, as a texture's key. */
  public readonly key: string;
  /** What samples it: the array as it is now. */
  public readonly target: ITextureTarget;

  private readonly prototype: CompressedTexture;
  private readonly limit: number;
  private readonly layers: Map<string, ITextureArrayLayer> = new Map();
  private readonly free: Array<number> = [];
  /** Keys whose layer is still to be copied. */
  private readonly copies: Set<string> = new Set();
  private texture: CompressedArrayTexture;
  private capacity: number;
  private used: number = 0;
  /** What the array was before it was replaced and before the frame copied it into the new one, or null. */
  private outgrown: Nullable<CompressedArrayTexture> = null;
  /** When a layer was last taken or the array last replaced, in milliseconds. */
  private changedAt: number = -Infinity;
  /** Layers of the outgrown array that hold anything, which is what is copied out of it. */
  private outgrownLayers: number = 0;
  /**
   * Arrays made and replaced again before a flush, which something may have bound meanwhile: let go with the flush's
   * other disposals, never at once.
   */
  private readonly discarded: Array<CompressedArrayTexture> = [];

  /**
   * @param key - Its key as a texture's.
   * @param prototype - A texture of its class, which it takes its format, size, levels and sampling from.
   * @param limit - Layers the device allows an array.
   */
  public constructor(key: string, prototype: CompressedTexture, limit: number) {
    this.key = key;
    this.prototype = prototype;
    this.limit = Math.max(1, limit);
    this.capacity = Math.min(INITIAL_LAYERS, this.limit);
    this.texture = this.createTexture(this.capacity);
    this.target = { value: this.texture };
  }

  /** Whether it has room for another key. */
  public get hasRoom(): boolean {
    return this.free.length > 0 || this.used < this.limit;
  }

  /** Whether it holds no key. */
  public get isEmpty(): boolean {
    return this.layers.size === 0;
  }

  /**
   * @param key - A texture's key, sampled from here by one more surface.
   * @param source - What the key holds on the GPU, of this array's class.
   * @returns Its layer, or null where the array is full.
   */
  public claim(key: string, source: Texture): Nullable<number> {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (held) {
      held.users += 1;

      return held.layer;
    }

    const layer: Nullable<number> = this.allocate();

    if (layer === null) {
      return null;
    }

    this.layers.set(key, { layer, source, users: 1 });
    this.copies.add(key);
    this.changedAt = performance.now();

    return layer;
  }

  /**
   * @param key - A texture's key one surface no longer samples from here.
   */
  public release(key: string): void {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (!held || --held.users > 0) {
      return;
    }

    this.layers.delete(key);
    this.copies.delete(key);
    this.free.push(held.layer);
  }

  /**
   * @param key - A key held here whose texture was replaced by another of the class, copied into its layer again.
   * @param source - What it holds now.
   */
  public refresh(key: string, source: Texture): void {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (held) {
      held.source = source;
      this.copies.add(key);
    }
  }

  /**
   * Fits the array to the layers it has used, once it has gone unchanged a while: a copy on the GPU, once a level.
   *
   * @param now - Milliseconds, on the clock claims are stamped by.
   */
  public compact(now: number): void {
    if (now - this.changedAt >= COMPACT_IDLE && this.used > 0 && this.used < this.capacity) {
      this.replace(this.used);
    }
  }

  /**
   * Says what the frame has to do for the array: move it into its replacement where it was replaced, then copy every
   * layer waiting.
   *
   * @returns The copies, and what to dispose once they are sent.
   */
  public flush(): ITextureArrayFlush {
    const flush: ITextureArrayFlush = { copies: [], disposals: [], evicted: [], replaced: [] };
    const outgrown: Nullable<CompressedArrayTexture> = this.outgrown;

    if (outgrown) {
      this.outgrown = null;

      if (this.outgrownLayers > 0) {
        this.pushCopies(flush, outgrown, 0, this.outgrownLayers, 0);
      }

      flush.disposals.push(outgrown);
      flush.replaced.push(this.key);
    }

    flush.disposals.push(...this.discarded);
    this.discarded.length = 0;

    for (const key of this.copies) {
      const held: ITextureArrayLayer = this.layers.get(key) as ITextureArrayLayer;

      this.pushCopies(flush, held.source, 0, 1, held.layer);
      // Its layer is the only copy anything samples from now on.
      flush.disposals.push(held.source);
      flush.evicted.push(key);
    }

    this.copies.clear();

    return flush;
  }

  public dispose(): void {
    this.outgrown?.dispose();
    this.discarded.forEach((texture: CompressedArrayTexture) => texture.dispose());
    this.texture.dispose();
    this.layers.clear();
    this.copies.clear();
  }

  /** A free layer, or one past those used, the array grown where it holds no more; null at the device's limit. */
  private allocate(): Nullable<number> {
    if (this.free.length) {
      return this.free.pop() as number;
    }

    if (this.used === this.limit) {
      return null;
    }

    if (this.used === this.capacity) {
      this.replace(Math.min(this.limit, Math.max(this.capacity + 1, Math.ceil(this.capacity * GROWTH))));
    }

    this.used += 1;

    return this.used - 1;
  }

  /** Makes the array anew at another size now, the layers held so far copied into it by the next flush. */
  private replace(capacity: number): void {
    // Replaced twice before a flush: the first array is still what holds the layers, and the second holds nothing.
    if (this.outgrown) {
      this.discarded.push(this.texture);
    } else {
      this.outgrown = this.texture;
      this.outgrownLayers = Math.min(this.used, this.capacity);
    }

    this.capacity = capacity;
    this.texture = this.createTexture(capacity);
    this.target.value = this.texture;
    this.changedAt = performance.now();
  }

  /** Copies of layers of every level, block rows whole, the last level's included however small. */
  private pushCopies(flush: ITextureArrayFlush, source: Texture, first: number, count: number, at: number): void {
    this.toExtents().forEach(({ width, height }: ILevelExtent, level: number) =>
      flush.copies.push({
        destination: this.texture,
        destinationLayer: at,
        height,
        layers: count,
        level,
        source,
        sourceLayer: first,
        width,
      })
    );
  }

  /** An array of its class holding `layers` layers, its first written as nothing and the rest left to the device. */
  private createTexture(layers: number): CompressedArrayTexture {
    const { prototype } = this;
    const { width, height } = prototype.image as { width: number; height: number };
    const extents: Array<ILevelExtent> = this.toExtents();
    const mipmaps = prototype.mipmaps.map((mipmap, level: number) => ({
      data: new Uint8Array(extents[level].bytes),
      height: (mipmap as { height: number }).height,
      width: (mipmap as { width: number }).width,
    }));
    const texture: CompressedArrayTexture = new CompressedArrayTexture(
      mipmaps as unknown as ConstructorParameters<typeof CompressedArrayTexture>[0],
      width,
      height,
      layers,
      prototype.format as CompressedPixelFormat
    );

    texture.colorSpace = prototype.colorSpace;
    texture.wrapS = prototype.wrapS;
    texture.wrapT = prototype.wrapT;
    texture.anisotropy = prototype.anisotropy;
    texture.magFilter = prototype.magFilter;
    texture.minFilter = prototype.minFilter;
    texture.generateMipmaps = false;
    // Three writes a layer from one buffer holding every layer; the first alone is written, from a buffer of one, and
    // the device clears the rest.
    texture.addLayerUpdate(0);
    texture.needsUpdate = true;

    return texture;
  }

  /** Each level's physical size, whole blocks, and its bytes a layer. */
  private toExtents(): Array<ILevelExtent> {
    const { width, height } = this.prototype.image as { width: number; height: number };
    const bytes: number = toBlockBytes(this.prototype.format as CompressedPixelFormat);

    return this.prototype.mipmaps.map((_, level: number) => {
      const blocksWide: number = Math.ceil(Math.max(1, width >> level) / BLOCK_SIDE);
      const blocksHigh: number = Math.ceil(Math.max(1, height >> level) / BLOCK_SIDE);

      return {
        bytes: blocksWide * blocksHigh * bytes,
        height: blocksHigh * BLOCK_SIDE,
        width: blocksWide * BLOCK_SIDE,
      };
    });
  }
}

/** Bytes a block of a format takes: eight for the one- and single-channel formats, sixteen for the rest. */
function toBlockBytes(format: CompressedPixelFormat): number {
  return format === RGBA_S3TC_DXT1_Format || format === RED_RGTC1_Format || format === SIGNED_RED_RGTC1_Format ? 8 : 16;
}
