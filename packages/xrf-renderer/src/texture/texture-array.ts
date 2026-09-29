import { Maybe, Nullable } from "@xrf/types";
import {
  CompressedArrayTexture,
  CompressedPixelFormat,
  CompressedTexture,
  CompressedTextureMipmap,
  MagnificationTextureFilter,
  MinificationTextureFilter,
  RED_RGTC1_Format,
  RGB_S3TC_DXT1_Format,
  RGBA_S3TC_DXT1_Format,
  SIGNED_RED_RGTC1_Format,
  Texture,
  WebGPURenderer,
  Wrapping,
} from "three/webgpu";

import { DDS_BLOCK_SIZE, DDS_FULL_BLOCK_BYTES, DDS_HALF_BLOCK_BYTES } from "#/dds/dds-block-format";
import { ITextureCopy } from "#/internals/texture-copy";
import { isTextureOnGpu } from "#/internals/texture-residency";
import { markRendererTextureNew } from "#/texture/renderer-texture-version";
import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { hasTextureData, releaseTextureData } from "#/texture/texture-data";
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

/** A level's texels a side, as its texture states them. */
interface ILevelSize {
  width: number;
  height: number;
}

/** What an array takes of the texture its class was first seen in: never the texture, whose bytes it would keep. */
interface IArrayShape {
  format: CompressedPixelFormat;
  width: number;
  height: number;
  levels: ReadonlyArray<ILevelSize>;
  colorSpace: string;
  wrapS: Wrapping;
  wrapT: Wrapping;
  anisotropy: number;
  magFilter: MagnificationTextureFilter;
  minFilter: MinificationTextureFilter;
}

/** Formats a block of which takes half the bytes of the others': one plane, or colour with a bit of alpha. */
const HALF_BLOCK_FORMATS: ReadonlySet<CompressedPixelFormat> = new Set<CompressedPixelFormat>([
  RGB_S3TC_DXT1_Format,
  RGBA_S3TC_DXT1_Format,
  RED_RGTC1_Format,
  SIGNED_RED_RGTC1_Format,
]);

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

  private readonly shape: IArrayShape;
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
    this.shape = toShape(prototype);
    this.zeros = zeros;
    this.extents = toExtents(this.shape);

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
   * @param source - What the key holds on the GPU, of this array's class; a layer still held for views drawn before is
   *   copied again from it.
   * @returns Its layer, or null where the array is full.
   */
  public claim(key: string, source: Texture): Nullable<number> {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (held) {
      held.users += 1;
      this.refresh(key, source);

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
   * @param key - A key held here that the surfaces claiming it from now on hold elsewhere: the views drawn before keep
   *   its layer as it is until they let it go, and nothing is copied into it again.
   */
  public detach(key: string): void {
    this.copies.delete(key);
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
   * @param key - A key held here whose own texture was let go.
   * @param destination - That texture, to be filled again on the GPU.
   * @returns The copies filling it from its layer, or null where the layer holds no copy of it: one of another texture,
   *   or one still to be made.
   */
  public toRestoreCopies(key: string, destination: Texture): Nullable<Array<ITextureCopy>> {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (!held || held.source !== destination || this.copies.has(key)) {
      return null;
    }

    // Until a flush moves them, the layers held before a replacement are the outgrown array's.
    const source: Texture = this.outgrown && held.layer < this.outgrownLayers ? this.outgrown : this.texture;

    return this.toCopies(source, held.layer, destination, 0, 1);
  }

  /**
   * @param key - A key held here.
   * @param copy - A copy the frame could not make.
   * @returns Whether it was a copy of the key's layer, which is copied again with the next flush.
   */
  public retry(key: string, copy: ITextureCopy): boolean {
    const held: Maybe<ITextureArrayLayer> = this.layers.get(key);

    if (
      !held ||
      copy.destination !== this.texture ||
      copy.destinationLayer !== held.layer ||
      copy.source !== held.source
    ) {
      return false;
    }

    this.copies.add(key);

    return true;
  }

  /**
   * Lets go of the zeroed layer the array was made with, once it is on the GPU: three never writes it again.
   *
   * @param renderer - The renderer that uploaded it.
   */
  public releaseZeros(renderer: WebGPURenderer): void {
    if (hasTextureData(this.texture) && isTextureOnGpu(renderer, this.texture)) {
      releaseTextureData(this.texture);
    }
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

  /** Copies of layers of every level into the array. */
  private pushCopies(flush: ITextureArrayFlush, source: Texture, first: number, count: number, at: number): void {
    flush.copies.push(...this.toCopies(source, first, this.texture, at, count));
  }

  /** Copies of layers of every level, block rows whole, the last level's included however small. */
  private toCopies(
    source: Texture,
    sourceLayer: number,
    destination: Texture,
    destinationLayer: number,
    layers: number
  ): Array<ITextureCopy> {
    return this.extents.map(({ width, height }: ILevelExtent, level: number): ITextureCopy => ({
      destination,
      destinationLayer,
      height,
      layers,
      level,
      source,
      sourceLayer,
      width,
    }));
  }

  /** An array of its class holding `layers` layers, its first written as nothing and the rest left to the device. */
  private createTexture(layers: number): CompressedArrayTexture {
    const { shape } = this;
    const mipmaps = shape.levels.map(({ width, height }: ILevelSize, level: number) => ({
      data: this.zeros(this.extents[level].bytes),
      height,
      width,
    }));
    const texture: CompressedArrayTexture = new CompressedArrayTexture(
      mipmaps as unknown as ConstructorParameters<typeof CompressedArrayTexture>[0],
      shape.width,
      shape.height,
      layers,
      shape.format
    );

    texture.colorSpace = shape.colorSpace;
    texture.wrapS = shape.wrapS;
    texture.wrapT = shape.wrapT;
    texture.anisotropy = shape.anisotropy;
    texture.magFilter = shape.magFilter;
    texture.minFilter = shape.minFilter;
    texture.generateMipmaps = false;
    // Three writes a layer from one buffer holding every layer; the first alone is written, from a buffer of one, and
    // the device clears the rest.
    texture.addLayerUpdate(0);
    markRendererTextureNew(texture);

    return texture;
  }
}

/**
 * @param prototype - A texture of the class, its bytes on the CPU or let go.
 * @returns What an array of the class is made from.
 */
function toShape(prototype: CompressedTexture): IArrayShape {
  const { width, height } = prototype.image as ILevelSize;

  return {
    anisotropy: prototype.anisotropy,
    colorSpace: prototype.colorSpace,
    format: prototype.format as CompressedPixelFormat,
    height,
    levels: prototype.mipmaps.map((mipmap: CompressedTextureMipmap) => {
      const level: ILevelSize = mipmap as unknown as ILevelSize;

      return { height: level.height, width: level.width };
    }),
    magFilter: prototype.magFilter,
    minFilter: prototype.minFilter,
    width,
    wrapS: prototype.wrapS,
    wrapT: prototype.wrapT,
  };
}

/**
 * @param shape - The class.
 * @returns Each level's physical size, whole blocks, and its bytes a layer.
 */
function toExtents(shape: IArrayShape): Array<ILevelExtent> {
  const blockBytes: number = HALF_BLOCK_FORMATS.has(shape.format) ? DDS_HALF_BLOCK_BYTES : DDS_FULL_BLOCK_BYTES;

  return shape.levels.map((_: ILevelSize, level: number) => {
    const width: number = Math.ceil(Math.max(1, shape.width >> level) / DDS_BLOCK_SIZE) * DDS_BLOCK_SIZE;
    const height: number = Math.ceil(Math.max(1, shape.height >> level) / DDS_BLOCK_SIZE) * DDS_BLOCK_SIZE;

    return { bytes: (width / DDS_BLOCK_SIZE) * (height / DDS_BLOCK_SIZE) * blockBytes, height, width };
  });
}
