import { Maybe, Nullable } from "@xrf/types";
import {
  CompressedArrayTexture,
  CompressedPixelFormat,
  CompressedTexture,
  CompressedTextureMipmap,
  Texture,
} from "three/webgpu";

import { DDS_BLOCK_SIZE } from "#/dds/dds-block-format";
import { ITextureCopy } from "#/internals/texture-copy";
import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { ITextureTarget } from "#/texture/texture-target";

/** Layers an array holds before it first grows. */
const INITIAL_LAYERS: number = 4;

/** How much an array grows by, which leaves at most a third of a grown one unused. */
const GROWTH: number = 1.5;

/** Milliseconds an array has to go unchanged before it is fitted: a level streaming in claims a layer at a time. */
const COMPACT_IDLE: number = 3000;

/**
 * Bytes an array holds at most, whatever the device allows: what one growth copies and holds twice while it does.
 * A class past it opens another array.
 */
export const TEXTURE_ARRAY_BYTES: number = 256 * 1024 * 1024;

/** One key's layer, and what it was last copied from, or is still to be. */
interface ITextureArrayLayer {
  layer: number;
  users: number;
  source: Texture;
}

/** A level's physical texels a side, whole blocks, and its bytes a layer. */
interface ILevelExtent {
  width: number;
  height: number;
  bytes: number;
}

/**
 * Textures of one class as the layers of one array, each copied on the GPU from its key's own texture, which then goes.
 * Grown by half again, to the device's layer limit or `TEXTURE_ARRAY_BYTES`, and fitted to the layers it uses once it
 * goes unchanged a while; a layer let go from the top is given up, and the lowest free layer is reused first, so the
 * top stays what can be given up.
 */
export class TextureArray {
  /** What the textures' users are told by when the array is replaced, as a texture's key. */
  public readonly key: string;
  /** What samples it: the array as it is now. */
  public readonly target: ITextureTarget;
  /** Layers it may hold at most: the device's limit, or what fits its bytes. */
  public readonly limit: number;

  private readonly prototype: CompressedTexture;
  private readonly extents: ReadonlyArray<ILevelExtent>;
  private readonly zeros: (bytes: number) => Uint8Array;
  private readonly layers: Map<string, ITextureArrayLayer> = new Map();
  /** Layers below `used` nothing holds. */
  private readonly free: Set<number> = new Set();
  /** Keys whose layer is still to be copied. */
  private readonly copies: Set<string> = new Set();
  private texture: CompressedArrayTexture;
  private capacity: number;
  /** Layers from the first up to the highest held. */
  private used: number = 0;
  /** What the array was before it was replaced and before the frame copied it into the new one, or null. */
  private outgrown: Nullable<CompressedArrayTexture> = null;
  /** Layers of the outgrown array that hold anything, which is what is copied out of it. */
  private outgrownLayers: number = 0;
  /**
   * Arrays made and replaced again before a flush, which something may have bound meanwhile: let go with the flush's
   * other disposals, never at once.
   */
  private readonly discarded: Array<CompressedArrayTexture> = [];
  /** When a layer was last taken or the array last replaced, in milliseconds. */
  private changedAt: number = -Infinity;

  /**
   * @param key - Its key as a texture's.
   * @param prototype - A texture of its class, which it takes its format, size, levels and sampling from.
   * @param limit - Layers the device allows an array.
   * @param zeros - A zeroed buffer of some bytes, shared by every array asking for as many: what a new array's first
   *   layer is written from.
   */
  public constructor(key: string, prototype: CompressedTexture, limit: number, zeros: (bytes: number) => Uint8Array) {
    this.key = key;
    this.prototype = prototype;
    this.zeros = zeros;
    this.extents = toExtents(prototype);

    const layerBytes: number = this.extents.reduce((total: number, extent: ILevelExtent) => total + extent.bytes, 0);

    this.limit = Math.max(1, Math.min(limit, Math.floor(TEXTURE_ARRAY_BYTES / Math.max(layerBytes, 1))));
    this.capacity = Math.min(INITIAL_LAYERS, this.limit);
    this.texture = this.createTexture(this.capacity);
    this.target = { value: this.texture };
  }

  /** Whether it has room for another key. */
  public get hasRoom(): boolean {
    return this.free.size > 0 || this.used < this.limit;
  }

  /** Whether it holds no key and waits on nothing, so it can go. */
  public get isEmpty(): boolean {
    return this.layers.size === 0 && this.outgrown === null && this.discarded.length === 0;
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
   * @param key - A texture's key, sampled from here by one more surface, whatever its own texture holds now.
   * @returns Whether it is held here, so the surface samples its layer.
   */
  public retain(key: string): boolean {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (held) {
      held.users += 1;
    }

    return Boolean(held);
  }

  /**
   * @param key - A texture's key one surface no longer samples from here.
   * @returns Whether no surface samples it any more, so its layer was given back.
   */
  public release(key: string): boolean {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (!held || --held.users > 0) {
      return false;
    }

    this.layers.delete(key);
    this.copies.delete(key);
    this.free.add(held.layer);

    // The top given up: what the array is fitted to shrinks with it.
    while (this.used > 0 && this.free.delete(this.used - 1)) {
      this.used -= 1;
    }

    return true;
  }

  /**
   * @param key - A key held here whose texture was replaced by another of the class, copied into its layer again.
   * @param source - What it holds now; the texture its layer was copied from, uploaded again, copies nothing.
   */
  public refresh(key: string, source: Texture): void {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (held && held.source !== source) {
      held.source = source;
      this.copies.add(key);
    }
  }

  /**
   * Fits the array to the layers it uses, once it has gone unchanged a while: a copy on the GPU, once a level.
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
      // Layers let go since it was outgrown are nothing's, and may lie past the end of an array fitted since.
      const layers: number = Math.min(this.outgrownLayers, this.used);

      this.outgrown = null;

      if (layers > 0) {
        this.pushCopies(flush, outgrown, 0, layers, 0);
      }

      flush.disposals.push(outgrown);
      flush.replaced.push(this.key);
    }

    flush.disposals.push(...this.discarded);
    this.discarded.length = 0;

    for (const key of this.copies) {
      const held: ITextureArrayLayer = this.layers.get(key) as ITextureArrayLayer;

      this.pushCopies(flush, held.source, 0, 1, held.layer);
      flush.evicted.push(key);
    }

    this.copies.clear();

    return flush;
  }

  /**
   * @param copy - A copy of a key's layer the frame could not make.
   * @returns The key, copied again with the next flush, or null for a copy of no key's layer here.
   */
  public retry(copy: ITextureCopy): Nullable<string> {
    if (copy.destination !== this.texture) {
      return null;
    }

    for (const [key, held] of this.layers) {
      if (held.layer === copy.destinationLayer && held.source === copy.source) {
        this.copies.add(key);

        return key;
      }
    }

    return null;
  }

  /** What to dispose where the array goes whole: itself, and anything it still waits to let go. */
  public listDisposals(): Array<Texture> {
    return [this.texture, ...(this.outgrown ? [this.outgrown] : []), ...this.discarded];
  }

  public dispose(): void {
    this.listDisposals().forEach((texture: Texture) => texture.dispose());
    this.outgrown = null;
    this.discarded.length = 0;
    this.layers.clear();
    this.copies.clear();
  }

  /** The lowest free layer, or one past those used, the array grown where it holds no more; null at its limit. */
  private allocate(): Nullable<number> {
    if (this.free.size) {
      const lowest: number = Math.min(...this.free);

      this.free.delete(lowest);

      return lowest;
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
    this.extents.forEach(({ width, height }: ILevelExtent, level: number) =>
      flush.copies.push({
        destination: this.texture,
        destinationLayer: at,
        height,
        layers: count,
        level,
        source,
        sourceLayer: first,
        width,
      } satisfies ITextureCopy)
    );
  }

  /** An array of its class holding `layers` layers, its first written as nothing and the rest left to the device. */
  private createTexture(layers: number): CompressedArrayTexture {
    const { prototype } = this;
    const { width, height } = prototype.image as { width: number; height: number };
    const mipmaps = prototype.mipmaps.map((mipmap: CompressedTextureMipmap, level: number) => ({
      data: this.zeros(this.extents[level].bytes),
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
}

/**
 * @param prototype - A texture of the class.
 * @returns Each level's physical size, whole blocks, and its bytes a layer, as its own data holds them.
 */
function toExtents(prototype: CompressedTexture): Array<ILevelExtent> {
  const { width, height } = prototype.image as { width: number; height: number };

  return prototype.mipmaps.map((mipmap: CompressedTextureMipmap, level: number) => ({
    bytes: (mipmap as { data: ArrayBufferView }).data.byteLength,
    height: Math.ceil(Math.max(1, height >> level) / DDS_BLOCK_SIZE) * DDS_BLOCK_SIZE,
    width: Math.ceil(Math.max(1, width >> level) / DDS_BLOCK_SIZE) * DDS_BLOCK_SIZE,
  }));
}
