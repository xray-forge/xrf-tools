import { Maybe, Nullable } from "@xrf/types";
import {
  BufferAttribute,
  BufferGeometry,
  InterleavedBufferAttribute,
  StorageBufferNode,
  TypedArray,
  WebGPURenderer,
} from "three/webgpu";

import { IClusterAttribute } from "#/geometry/cluster-attribute";
import { IClusterSource } from "#/geometry/cluster-source";
import { EClusterWordFormat } from "#/geometry/cluster-word-format";
import { isPackedTreeGeometry } from "#/geometry/packed-tree-geometry";
import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { StaticArenaBuffer } from "#/scene/static/static-arena-buffer";
import { toFittedCapacity } from "#/scene/static/static-growth";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRoom } from "#/scene/static/static-room";
import { StorageRetirement } from "#/uniforms/storage-retirement";

/** Vertices an arena starts with. */
const INITIAL_VERTICES: number = 1 << 16;
/** Indices an arena starts with. */
const INITIAL_INDICES: number = 1 << 18;

/** Nothing still to be placed. */
const NOTHING_COMING: IStaticRoom = { indices: 0, vertices: 0 };

type TTypedArrayConstructor = new (length: number) => TypedArray;

/** Every arena made, numbered: the number marks the programs built for it, which read its buffers by name. */
let arenaCount: number = 0;

/**
 * One buffer of words holding every static geometry of a vertex layout, a vertex its attributes one after another, and
 * one of indices: every static draw of that layout reads its vertices from them by what its cluster names, so every
 * static draw of a material over the layout is drawn by one object. Both live on the GPU alone: a geometry placed is
 * sent with the next flush, and a growth copies a buffer into a larger one there, the nodes every shader reads them
 * through pointed at the new ones.
 */
export class StaticArena implements IClusterSource {
  /**
   * @param buffer - A geometry's buffers.
   * @returns The layout they are stored in, which an arena holds geometries of one of; null for one holding an
   *   attribute an arena stores no word format for, which draws plainly.
   */
  public static toSignature(buffer: BufferGeometry): Nullable<string> {
    const attributes: Nullable<Array<IClusterAttribute>> = StaticArena.toAttributes(buffer);

    return attributes
      ? attributes
          .map(({ name, type, itemSize, isNormalized }: IClusterAttribute) =>
            [name, type.name, itemSize, isNormalized ? "normalized" : ""].join(":")
          )
          .join(",")
      : null;
  }

  /** Each attribute's place among a vertex's words, sorted by name; null where one has no word format. */
  private static toAttributes(buffer: BufferGeometry): Nullable<Array<IClusterAttribute>> {
    const attributes: Array<IClusterAttribute> = [];
    let offset: number = 0;

    for (const [name, attribute] of Object.entries(buffer.attributes).sort(
      (
        [left]: [string, BufferAttribute | InterleavedBufferAttribute],
        [right]: [string, BufferAttribute | InterleavedBufferAttribute]
      ) => left.localeCompare(right)
    )) {
      // Read only by water, which draws plainly: an arena stores it for nothing, and would split its layouts by it.
      if (name === EVertexAttribute.PACKED_COLOR) {
        continue;
      }

      const { array, itemSize, normalized } = attribute as BufferAttribute;
      let format: Maybe<EClusterWordFormat>;
      let words: number = itemSize;

      if (array instanceof Float32Array) {
        format = EClusterWordFormat.FLOAT;
      } else if (array instanceof Uint32Array) {
        format = EClusterWordFormat.UINT;
      } else if (array instanceof Uint8Array && normalized && itemSize === 4) {
        format = EClusterWordFormat.UNORM8X4;
        words = 1;
      }

      if (!format) {
        return null;
      }

      attributes.push({
        format,
        isNormalized: normalized,
        itemSize,
        name,
        offset,
        type: array.constructor as TTypedArrayConstructor,
        words,
      });
      offset += words;
    }

    return attributes;
  }

  /** Its number, which marks the programs built for it. */
  public readonly id: number = ++arenaCount;
  public readonly signature: string;
  /** Each attribute's place among a vertex's words. */
  public readonly layout: ReadonlyArray<IClusterAttribute>;
  /** Words a vertex takes. */
  public readonly stride: number;
  /**
   * A geometry of three vertices in its layout, marked for its arena: what a material compiles against for its static
   * draws and what every batch over the arena is made of, never drawn from, and never replaced as the arena grows.
   */
  public readonly prototype: BufferGeometry;
  /** Whether its geometry is a tree's, packed with the rigidity the wind sways it by. */
  public readonly isSwaying: boolean;
  public readonly entryNode: StorageBufferNode<"uvec2">;
  public readonly rangeNode: StorageBufferNode<"uvec4">;

  private readonly vertices: RangeAllocator = new RangeAllocator();
  private readonly indices: RangeAllocator = new RangeAllocator();
  private readonly words: StaticArenaBuffer;
  private readonly index: StaticArenaBuffer;
  private currentGeneration: number = 0;

  /**
   * @param buffer - A geometry whose layout the arena holds, which has one (`toSignature`).
   * @param entryNode - Every view's kept clusters, which a clustered draw's instances are.
   * @param rangeNode - Every cluster's range, which an entry names.
   * @param retirement - Where the buffers a growth replaces go.
   */
  public constructor(
    buffer: BufferGeometry,
    entryNode: StorageBufferNode<"uvec2">,
    rangeNode: StorageBufferNode<"uvec4">,
    retirement: StorageRetirement
  ) {
    this.entryNode = entryNode;
    this.rangeNode = rangeNode;
    this.layout = StaticArena.toAttributes(buffer) as Array<IClusterAttribute>;
    this.signature = StaticArena.toSignature(buffer) as string;
    this.stride = this.layout.reduce((total: number, attribute: IClusterAttribute) => total + attribute.words, 0);
    this.isSwaying = isPackedTreeGeometry(buffer);
    this.words = new StaticArenaBuffer(retirement);
    this.index = new StaticArenaBuffer(retirement);
    this.prototype = this.createPrototype();
  }

  /** The words as the node every shader over the arena reads them through. */
  public get wordNode(): StorageBufferNode<"uint"> {
    return this.words.node;
  }

  /** The indices, likewise. */
  public get indexNode(): StorageBufferNode<"uint"> {
    return this.index.node;
  }

  /** Bumped whenever the arena's buffers are replaced, which happens as it flushes. */
  public get generation(): number {
    return this.currentGeneration;
  }

  /**
   * Copies a geometry in, growing the arena once where it does not fit: for everything still to come where that is
   * known, since every growth copies the whole arena again on the GPU, and only the buffer that is short. What it
   * copies in waits on the CPU for the next flush.
   *
   * @param buffer - A geometry in the arena's layout.
   * @param toComing - The room the geometries still to be placed after it will take, asked only when the arena grows.
   * @param limits - The most vertices and indices the device lets a buffer of the arena hold.
   * @returns Where it sits, or null where the arena cannot grow to hold it, nothing grown.
   */
  public place(buffer: BufferGeometry, toComing: () => IStaticRoom, limits: IStaticRoom): Nullable<IStaticRange> {
    const vertexCount: number = buffer.getAttribute("position").count;
    const index: ArrayLike<number> = buffer.index?.array ?? StaticArena.createSequence(vertexCount);
    const isShort: boolean = !this.vertices.fits(vertexCount) || !this.indices.fits(index.length);
    const coming: IStaticRoom = isShort ? toComing() : NOTHING_COMING;
    const vertices: Nullable<number> = toFittedCapacity(
      this.vertices,
      vertexCount,
      INITIAL_VERTICES,
      limits.vertices,
      () => coming.vertices
    );
    const indices: Nullable<number> = toFittedCapacity(
      this.indices,
      index.length,
      INITIAL_INDICES,
      limits.indices,
      () => coming.indices
    );

    if (vertices === null || indices === null) {
      return null;
    }

    if (vertices > this.vertices.capacity) {
      this.vertices.grow(vertices);
      this.words.reserve(vertices * this.stride);
    }

    if (indices > this.indices.capacity) {
      this.indices.grow(indices);
      this.index.reserve(indices);
    }

    // Both fit now, as sized.
    const vertexStart: number = this.vertices.allocate(vertexCount) ?? 0;
    const indexStart: number = this.indices.allocate(index.length) ?? 0;

    this.words.write(vertexStart * this.stride, this.toWords(buffer, vertexCount));
    // A geometry's own indices are sent as they are, since nothing writes them: held until the flush, not copied.
    this.index.write(indexStart, index instanceof Uint32Array ? index : Uint32Array.from(index));

    return { arena: this, indexCount: index.length, indexStart, vertexCount, vertexStart };
  }

  /**
   * Sends every geometry placed since the last flush, each buffer grown on the GPU first where it was asked to hold
   * more. Before the frame culls, records or draws anything over the arena.
   *
   * @param renderer - The renderer drawing.
   */
  public flush(renderer: WebGPURenderer): void {
    // Both, whichever grew.
    const isWordsReplaced: boolean = this.words.flush(renderer);
    const isIndexReplaced: boolean = this.index.flush(renderer);

    if (isWordsReplaced || isIndexReplaced) {
      this.currentGeneration += 1;
    }
  }

  /** The words and indices placed that wait for the next flush, which the CPU holds until then. */
  public listPending(): Array<Uint32Array> {
    return [...this.words.listPending(), ...this.index.listPending()];
  }

  /**
   * @param range - A geometry placed, whose room another can take from now on.
   */
  public free(range: IStaticRange): void {
    this.vertices.release(range.vertexStart, range.vertexCount);
    this.indices.release(range.indexStart, range.indexCount);
  }

  /**
   * @returns A geometry over no data in the arena's layout, for one object to draw its batch's clusters from: the
   *   prototype's attributes, which nothing reads, keying the programs it is drawn with as the prototype does.
   */
  public createGeometry(): BufferGeometry {
    const geometry: BufferGeometry = new BufferGeometry();

    for (const [name, attribute] of Object.entries(this.prototype.attributes)) {
      geometry.setAttribute(name, attribute);
    }

    geometry.userData = this.prototype.userData;

    return geometry;
  }

  /** Gives its buffers up, for them to go once nothing binds them. */
  public dispose(): void {
    this.prototype.dispose();
    this.words.dispose();
    this.index.dispose();
  }

  private static createSequence(count: number): Uint32Array {
    const sequence: Uint32Array = new Uint32Array(count);

    for (let it = 0; it < count; it += 1) {
      sequence[it] = it;
    }

    return sequence;
  }

  /** A geometry's vertices as the arena stores them: each attribute's words of every vertex, one after another. */
  private toWords(buffer: BufferGeometry, vertexCount: number): Uint32Array {
    const words: Uint32Array = new Uint32Array(vertexCount * this.stride);
    const floats: Float32Array = new Float32Array(words.buffer);

    for (const { name, format, itemSize, offset } of this.layout) {
      const source: TypedArray = buffer.getAttribute(name).array as TypedArray;
      const target: Uint32Array | Float32Array = format === EClusterWordFormat.FLOAT ? floats : words;
      // Four normalized bytes are one word, read as the word they fill.
      const values: TypedArray =
        format === EClusterWordFormat.UNORM8X4
          ? new Uint32Array(source.buffer, source.byteOffset, vertexCount)
          : source;
      const width: number = format === EClusterWordFormat.UNORM8X4 ? 1 : itemSize;

      for (let vertex = 0; vertex < vertexCount; vertex += 1) {
        const at: number = vertex * this.stride + offset;

        for (let component = 0; component < width; component += 1) {
          target[at + component] = values[vertex * width + component];
        }
      }
    }

    return words;
  }

  /** Three vertices in the arena's layout, and the mark of the arena, which programs built for it are keyed by. */
  private createPrototype(): BufferGeometry {
    const geometry: BufferGeometry = new BufferGeometry();

    for (const { name, type, itemSize, isNormalized } of this.layout) {
      geometry.setAttribute(name, new BufferAttribute(new type(3 * itemSize), itemSize, isNormalized));
    }

    // Never read, so never bound: the name alone keys the programs, which read this arena's buffers by name.
    geometry.setAttribute(`${EVertexAttribute.CLUSTER_ARENA}${this.id}`, new BufferAttribute(new Uint32Array(3), 1));
    geometry.setDrawRange(0, 0);
    geometry.userData = { clusters: this };

    return geometry;
  }
}
