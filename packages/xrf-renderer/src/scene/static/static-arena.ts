import { Maybe, Nullable } from "@xrf/types";
import {
  BufferAttribute,
  BufferGeometry,
  InterleavedBufferAttribute,
  StorageBufferAttribute,
  StorageBufferNode,
  TypedArray,
} from "three/webgpu";

import { IClusterAttribute } from "#/geometry/cluster-attribute";
import { IClusterSource } from "#/geometry/cluster-source";
import { EClusterWordFormat } from "#/geometry/cluster-word-format";
import { isPackedTreeGeometry } from "#/geometry/packed-tree-geometry";
import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { queueBufferUpload } from "#/scene/buffer-upload";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { createArenaNode } from "#/scene/static/static-arena-nodes.tsl";
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
 * static draw of a material over the layout is drawn by one object. It grows by replacing its buffers, the nodes every
 * shader reads them through pointed at the new ones.
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
  /** The words as the node every shader over the arena reads them through. */
  public readonly wordNode: StorageBufferNode<"uint">;
  /** The indices, likewise. */
  public readonly indexNode: StorageBufferNode<"uint">;

  private readonly vertices: RangeAllocator = new RangeAllocator();
  private readonly indices: RangeAllocator = new RangeAllocator();
  private readonly retirement: StorageRetirement;
  private words: StorageBufferAttribute;
  private index: StorageBufferAttribute;
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
    this.retirement = retirement;
    this.entryNode = entryNode;
    this.rangeNode = rangeNode;
    this.layout = StaticArena.toAttributes(buffer) as Array<IClusterAttribute>;
    this.signature = StaticArena.toSignature(buffer) as string;
    this.stride = this.layout.reduce((total: number, attribute: IClusterAttribute) => total + attribute.words, 0);
    this.isSwaying = isPackedTreeGeometry(buffer);
    this.words = new StorageBufferAttribute(new Uint32Array(this.stride), 1);
    this.index = new StorageBufferAttribute(new Uint32Array(1), 1);
    this.wordNode = createArenaNode(this.words);
    this.indexNode = createArenaNode(this.index);
    this.prototype = this.createPrototype();
  }

  /** Bumped whenever the arena's buffers are replaced. */
  public get generation(): number {
    return this.currentGeneration;
  }

  /**
   * Copies a geometry in, growing the arena once where it does not fit: for everything still to come where that is
   * known, since every growth copies and uploads the whole arena again, and only the buffer that is short.
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

    if (vertices > this.vertices.capacity || indices > this.indices.capacity) {
      this.grow(vertices, indices);
    }

    // Both fit now, as sized.
    const vertexStart: number = this.vertices.allocate(vertexCount) ?? 0;
    const indexStart: number = this.indices.allocate(index.length) ?? 0;

    this.writeVertices(buffer, vertexStart, vertexCount);
    (this.index.array as Uint32Array).set(index, indexStart);
    queueBufferUpload(this.index, indexStart, index.length);

    return { arena: this, indexCount: index.length, indexStart, vertexCount, vertexStart };
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
    this.retirement.retire([this.words, this.index]);
  }

  private static createSequence(count: number): Uint32Array {
    const sequence: Uint32Array = new Uint32Array(count);

    for (let it = 0; it < count; it += 1) {
      sequence[it] = it;
    }

    return sequence;
  }

  /** Copies each attribute of a geometry into its words of every vertex. */
  private writeVertices(buffer: BufferGeometry, vertexStart: number, vertexCount: number): void {
    const words: Uint32Array = this.words.array as Uint32Array;
    const floats: Float32Array = new Float32Array(words.buffer, words.byteOffset, words.length);

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
        const at: number = (vertexStart + vertex) * this.stride + offset;

        for (let component = 0; component < width; component += 1) {
          target[at + component] = values[vertex * width + component];
        }
      }
    }

    queueBufferUpload(this.words, vertexStart * this.stride, vertexCount * this.stride);
  }

  /** Buffers of the sizes given, holding everything the current ones do, uploaded whole with their next use. */
  private grow(vertices: number, indices: number): void {
    if (vertices > this.vertices.capacity) {
      this.words = this.replace(this.words, vertices * this.stride);
      this.wordNode.value = this.words;
      this.vertices.grow(vertices);
    }

    if (indices > this.indices.capacity) {
      this.index = this.replace(this.index, indices);
      this.indexNode.value = this.index;
      this.indices.grow(indices);
    }

    this.currentGeneration += 1;
  }

  /** A buffer of the length given holding everything the one it replaces does, which is retired. */
  private replace(attribute: StorageBufferAttribute, length: number): StorageBufferAttribute {
    const array: Uint32Array = new Uint32Array(length);

    array.set(attribute.array as Uint32Array);
    this.retirement.retire([attribute]);

    return new StorageBufferAttribute(array, 1);
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
