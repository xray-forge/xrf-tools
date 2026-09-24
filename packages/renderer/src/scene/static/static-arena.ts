import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, BufferGeometry, InstancedBufferAttribute, TypedArray } from "three/webgpu";

import { RangeAllocator } from "#/scene/static/range-allocator";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { STATIC_HEADROOM, toGrownCapacity } from "#/scene/static/static-growth";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRoom } from "#/scene/static/static-room";
import { EVertexAttribute } from "#/shader/vertex-attribute";

/** Vertices an arena starts with. */
const INITIAL_VERTICES: number = 1 << 16;
/** Indices an arena starts with. */
const INITIAL_INDICES: number = 1 << 18;
/** Vertices an arena grows to at most: its widest attribute, four floats, then fills half a WebGPU buffer's default limit. */
const VERTEX_LIMIT: number = 1 << 23;
/** Indices an arena grows to at most, half a WebGPU buffer's default limit. */
const INDEX_LIMIT: number = 1 << 25;

type TTypedArrayConstructor = new (length: number) => TypedArray;

/** One vertex attribute as every geometry in an arena stores it. */
interface IArenaAttribute {
  name: string;
  type: TTypedArrayConstructor;
  itemSize: number;
  isNormalized: boolean;
}

/**
 * One vertex and one index buffer holding every static geometry of a vertex layout, so every static draw of a material
 * over that layout is drawn by one object. It grows by replacing its buffers, which every geometry drawing them
 * has to be made again for: its generation says when.
 */
export class StaticArena {
  /**
   * @param buffer - A geometry's buffers.
   * @returns The layout they are stored in, which an arena holds geometries of one of.
   */
  public static toSignature(buffer: BufferGeometry): string {
    return StaticArena.toAttributes(buffer)
      .map(({ name, type, itemSize, isNormalized }: IArenaAttribute) =>
        [name, type.name, itemSize, isNormalized ? "normalized" : ""].join(":")
      )
      .join(",");
  }

  private static toAttributes(buffer: BufferGeometry): Array<IArenaAttribute> {
    return Object.entries(buffer.attributes)
      .map(([name, attribute]) => ({
        isNormalized: attribute.normalized,
        itemSize: attribute.itemSize,
        name,
        type: attribute.array.constructor as TTypedArrayConstructor,
      }))
      .sort((left: IArenaAttribute, right: IArenaAttribute) => left.name.localeCompare(right.name));
  }

  /**
   * Gives a geometry the attribute its kind of static draw is built for: the slot attribute a single draw reads, or
   * the mark an instanced one is known by, which its shader never reads and so never binds.
   */
  private static mark(geometry: BufferGeometry, kind: EStaticDrawKind, slots: InstancedBufferAttribute): void {
    if (kind === EStaticDrawKind.SINGLE) {
      geometry.setAttribute(EVertexAttribute.STATIC_SLOT, slots);
    } else {
      geometry.setAttribute(EVertexAttribute.INSTANCE_LIST, new InstancedBufferAttribute(new Uint32Array(1), 1));
    }
  }

  private static createSlots(count: number): InstancedBufferAttribute {
    return new InstancedBufferAttribute(StaticArena.createSequence(count), 1);
  }

  private static createSequence(count: number): Uint32Array {
    const sequence: Uint32Array = new Uint32Array(count);

    for (let it = 0; it < count; it += 1) {
      sequence[it] = it;
    }

    return sequence;
  }

  public readonly signature: string;
  /**
   * A geometry of three vertices in its layout for each kind of static draw, with the attribute that kind is built
   * for: what a material compiles against for its static draws, never drawn, and never replaced as the arena grows.
   */
  public readonly prototypes: Record<EStaticDrawKind, BufferGeometry>;

  private readonly layout: ReadonlyArray<IArenaAttribute>;
  private readonly vertices: RangeAllocator = new RangeAllocator();
  private readonly indices: RangeAllocator = new RangeAllocator();
  private attributes: Map<string, BufferAttribute> = new Map();
  private index: BufferAttribute = new BufferAttribute(new Uint32Array(0), 1);
  /** Slots the static draw buffers hold, which its slot attribute numbers. */
  private slotCount: number;
  /** Every slot's own number, read by a static draw's first instance: which slot it draws. */
  private slots: InstancedBufferAttribute;
  private currentGeneration: number = 0;
  private placed: number = 0;

  /**
   * @param buffer - A geometry whose layout the arena holds.
   * @param slots - Slots the static draw buffers hold.
   */
  public constructor(buffer: BufferGeometry, slots: number) {
    this.slotCount = slots;
    this.slots = StaticArena.createSlots(slots);
    this.signature = StaticArena.toSignature(buffer);
    this.layout = StaticArena.toAttributes(buffer);
    this.prototypes = {
      [EStaticDrawKind.SINGLE]: this.createPrototype(EStaticDrawKind.SINGLE),
      [EStaticDrawKind.LISTED]: this.createPrototype(EStaticDrawKind.LISTED),
    };
  }

  /** Bumped whenever the arena's buffers are replaced. */
  public get generation(): number {
    return this.currentGeneration;
  }

  /** Whether no geometry is placed in it. */
  public get isEmpty(): boolean {
    return this.placed === 0;
  }

  /**
   * Copies a geometry in, growing the arena where it does not fit: once for everything still to come where that is
   * known, since every growth copies and uploads the whole arena again.
   *
   * @param buffer - A geometry in the arena's layout.
   * @param toComing - The room the geometries still to be placed after it will take, asked only when the arena grows.
   * @returns Where it sits, or null where the arena cannot grow to hold it.
   */
  public place(buffer: BufferGeometry, toComing: () => IStaticRoom): Nullable<IStaticRange> {
    const vertexCount: number = buffer.getAttribute("position").count;
    const index: ArrayLike<number> = buffer.index?.array ?? StaticArena.createSequence(vertexCount);

    if (!this.vertices.capacity || !this.fits(vertexCount, index.length)) {
      const coming: IStaticRoom = toComing();

      // Its buffers are made with the first geometry placed: a material compiles against its prototype alone.
      this.grow(
        StaticArena.toCapacity(this.vertices, vertexCount + coming.vertices, INITIAL_VERTICES, VERTEX_LIMIT),
        StaticArena.toCapacity(this.indices, index.length + coming.indices, INITIAL_INDICES, INDEX_LIMIT)
      );
    }

    const vertexStart: Nullable<number> = this.allocate(this.vertices, vertexCount, VERTEX_LIMIT);
    const indexStart: Nullable<number> =
      vertexStart === null ? null : this.allocate(this.indices, index.length, INDEX_LIMIT);

    if (vertexStart === null || indexStart === null) {
      if (vertexStart !== null) {
        this.vertices.release(vertexStart, vertexCount);
      }

      return null;
    }

    for (const { name, itemSize } of this.layout) {
      const attribute: BufferAttribute = this.attributes.get(name) as BufferAttribute;

      attribute.array.set(buffer.getAttribute(name).array as TypedArray, vertexStart * itemSize);
      attribute.addUpdateRange(vertexStart * itemSize, vertexCount * itemSize);
      attribute.needsUpdate = true;
    }

    this.index.array.set(index, indexStart);
    this.index.addUpdateRange(indexStart, index.length);
    this.index.needsUpdate = true;
    this.placed += 1;

    return { arena: this, indexCount: index.length, indexStart, vertexCount, vertexStart };
  }

  /**
   * Numbers every slot of static draw buffers that grew: a slot attribute of the new count, which every geometry
   * drawing the arena is made again for.
   *
   * @param slots - Slots the buffers hold from now on.
   */
  public growSlots(slots: number): void {
    this.slotCount = slots;
    // The old one is freed with the geometries drawing it, which its generation has made again.
    this.slots = StaticArena.createSlots(slots);
    this.currentGeneration += 1;
  }

  /**
   * @param range - A geometry placed, whose room another can take from now on.
   */
  public free(range: IStaticRange): void {
    this.vertices.release(range.vertexStart, range.vertexCount);
    this.indices.release(range.indexStart, range.indexCount);
    this.placed -= 1;
  }

  /**
   * @param kind - The kind of static draw it draws.
   * @returns A geometry over the arena's buffers as they are now, for one object to draw many static draws of.
   *   Disposing it frees those buffers, which is only for when the arena grows or goes.
   */
  public createGeometry(kind: EStaticDrawKind): BufferGeometry {
    const geometry: BufferGeometry = new BufferGeometry();

    this.attributes.forEach((attribute: BufferAttribute, name: string) => geometry.setAttribute(name, attribute));
    StaticArena.mark(geometry, kind, this.slots);
    geometry.setIndex(this.index);

    return geometry;
  }

  public dispose(): void {
    Object.values(this.prototypes).forEach((prototype: BufferGeometry) => prototype.dispose());
  }

  /** Whether both runs fit as the arena stands, without growing it. */
  private fits(vertexCount: number, indexCount: number): boolean {
    const vertexStart: Nullable<number> = this.vertices.allocate(vertexCount);

    if (vertexStart === null) {
      return false;
    }

    this.vertices.release(vertexStart, vertexCount);

    const indexStart: Nullable<number> = this.indices.allocate(indexCount);

    if (indexStart === null) {
      return false;
    }

    this.indices.release(indexStart, indexCount);

    return true;
  }

  /**
   * @returns What a buffer grows to so that `wanted` more elements fit beside what it holds, with headroom: at least
   *   its initial size, more than it was, and never past its limit.
   */
  private static toCapacity(allocator: RangeAllocator, wanted: number, initial: number, limit: number): number {
    return toGrownCapacity(allocator.used, wanted, allocator.capacity, initial, limit);
  }

  /** A run of a buffer's elements, the buffers grown until it fits or the limit says it never will. */
  private allocate(allocator: RangeAllocator, count: number, limit: number): Nullable<number> {
    let start: Nullable<number> = allocator.allocate(count);

    while (start === null && allocator.capacity < limit) {
      // Room freed in runs too short for it: grown past the whole run it needs.
      const capacity: number = Math.min(limit, Math.ceil((allocator.capacity + count) * STATIC_HEADROOM));

      if (allocator === this.vertices) {
        this.grow(capacity, this.indices.capacity);
      } else {
        this.grow(this.vertices.capacity, capacity);
      }

      start = allocator.allocate(count);
    }

    return start;
  }

  /** Buffers of the sizes given, holding everything the current ones do. */
  private grow(vertices: number, indices: number): void {
    const attributes: Map<string, BufferAttribute> = new Map();

    for (const { name, type, itemSize, isNormalized } of this.layout) {
      const array: TypedArray = new type(vertices * itemSize);
      const previous: Maybe<BufferAttribute> = this.attributes.get(name);

      if (previous) {
        array.set(previous.array);
      }

      attributes.set(name, new BufferAttribute(array, itemSize, isNormalized));
    }

    const index: Uint32Array = new Uint32Array(indices);

    index.set(this.index.array);
    this.attributes = attributes;
    this.index = new BufferAttribute(index, 1);
    // Freed with the geometries drawing it; a new one is uploaded with the ones that replace them.
    this.slots = StaticArena.createSlots(this.slotCount);
    this.vertices.grow(vertices);
    this.indices.grow(indices);
    this.currentGeneration += 1;
  }

  private createPrototype(kind: EStaticDrawKind): BufferGeometry {
    const geometry: BufferGeometry = new BufferGeometry();

    for (const { name, type, itemSize, isNormalized } of this.layout) {
      geometry.setAttribute(name, new BufferAttribute(new type(3 * itemSize), itemSize, isNormalized));
    }

    StaticArena.mark(geometry, kind, StaticArena.createSlots(1));
    geometry.setIndex(new BufferAttribute(new Uint32Array([0, 1, 2]), 1));

    return geometry;
  }
}
