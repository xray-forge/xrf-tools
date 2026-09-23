import { Nullable } from "@xrf/types";
import { BufferAttribute, IndirectStorageBufferAttribute, Matrix4, Sphere } from "three/webgpu";

import {
  STATIC_DRAW_ARGUMENTS,
  STATIC_DRAW_CAPACITY,
  STATIC_ROW_CAPACITY,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/** The slots written since the buffers last went up, as one span. */
interface IDirtySpan {
  first: number;
  last: number;
}

/**
 * Hands out the slots of the static draw buffers and writes what each draws, uploading the slots changed in a frame
 * as one span a buffer.
 */
export class StaticDrawPool {
  /**
   * Whether static draws are drawn at all: only on a device drawing an indirect draw's first instance, which is how a
   * static draw finds its slot.
   */
  public isEnabled: boolean = false;

  private readonly buffers: StaticDrawBuffers;
  private readonly free: Array<number> = [];
  /** Slots handed out at least once; the cull only reads below it. */
  private used: number = 0;
  private readonly dirty: IDirtySpan = { first: Infinity, last: -1 };
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
  }

  /** The indirect arguments every static draw is drawn by, as the first cull leaves them. */
  public get args(): IndirectStorageBufferAttribute {
    return this.buffers.args;
  }

  /** The indirect arguments every static draw is drawn by again, as the second cull leaves them. */
  public get lateArgs(): IndirectStorageBufferAttribute {
    return this.buffers.lateArgs;
  }

  /** Bumped whenever a slot changes what it draws or where, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  /** Slots in use, a draw each. */
  public get count(): number {
    return this.used - this.free.length;
  }

  /**
   * @returns A slot, or null where every slot is taken and the draw has to be drawn otherwise.
   */
  public allocate(): Nullable<number> {
    if (!this.isEnabled) {
      return null;
    }

    if (this.free.length) {
      return this.free.pop() as number;
    }

    if (this.used === STATIC_DRAW_CAPACITY) {
      return null;
    }

    this.used += 1;

    return this.used - 1;
  }

  /**
   * @param slot - The slot drawing, which is also its first instance: the element of its arena's slot attribute
   *   holding its own number.
   * @param start - Its first index, in its arena.
   * @param count - Indices it draws; none draws nothing.
   * @param baseVertex - Where its geometry's vertices start in its arena.
   * @param sphere - What it spans in renderer space.
   * @param matrix - Where it stands.
   */
  public write(slot: number, start: number, count: number, baseVertex: number, sphere: Sphere, matrix: Matrix4): void {
    const args = this.buffers.args.array as Uint32Array;
    const at: number = slot * STATIC_DRAW_ARGUMENTS;

    // The instance count is the cull's to write; one until it has.
    args[at] = count;
    args[at + 1] = 1;
    args[at + 2] = start;
    args[at + 3] = baseVertex;
    args[at + 4] = slot;
    this.copyLate(at, 0);
    (this.buffers.spheres.array as Float32Array).set(
      [sphere.center.x, sphere.center.y, sphere.center.z, count ? sphere.radius : -1],
      slot * 4
    );
    (this.buffers.models.array as Float32Array).set(matrix.elements, slot * 16);
    this.touch(slot);
  }

  /**
   * @param slot - The slot drawing an instanced draw, whose instance count the instance cull writes.
   * @param start - Its first index, in its arena.
   * @param count - Indices it draws.
   * @param baseVertex - Where its geometry's vertices start in its arena.
   * @param firstInstance - Where its list of kept places starts.
   */
  public writeListed(slot: number, start: number, count: number, baseVertex: number, firstInstance: number): void {
    const args = this.buffers.args.array as Uint32Array;
    const at: number = slot * STATIC_DRAW_ARGUMENTS;

    args[at] = count;
    args[at + 1] = 0;
    args[at + 2] = start;
    args[at + 3] = baseVertex;
    args[at + 4] = firstInstance;
    // The second cull lists its places in the second half of the list.
    this.copyLate(at, STATIC_ROW_CAPACITY);
    // Culled by its rows, never as a slot: the slot cull leaves its instance count at none for the rows to count up.
    (this.buffers.spheres.array as Float32Array)[slot * 4 + 3] = -1;
    this.touch(slot);
  }

  /**
   * @param slot - A slot drawing nothing from now on, free for another draw.
   */
  public release(slot: number): void {
    (this.buffers.args.array as Uint32Array)[slot * STATIC_DRAW_ARGUMENTS + 1] = 0;
    (this.buffers.lateArgs.array as Uint32Array)[slot * STATIC_DRAW_ARGUMENTS + 1] = 0;
    (this.buffers.spheres.array as Float32Array)[slot * 4 + 3] = -1;
    this.free.push(slot);
    this.touch(slot);
  }

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public flush(): void {
    if (this.dirty.last < this.dirty.first) {
      return;
    }

    const { first, last } = this.dirty;

    StaticDrawPool.upload(this.buffers.args, first, last, STATIC_DRAW_ARGUMENTS);
    StaticDrawPool.upload(this.buffers.lateArgs, first, last, STATIC_DRAW_ARGUMENTS);
    StaticDrawPool.upload(this.buffers.spheres, first, last, 4);
    StaticDrawPool.upload(this.buffers.models, first, last, 16);
    this.dirty.first = Infinity;
    this.dirty.last = -1;
  }

  /** Slots the cull has to look at: every slot ever handed out. */
  public get extent(): number {
    return this.used;
  }

  /** A slot's arguments for the second draw: the first's, with no instances and the list moved on by `offset`. */
  private copyLate(at: number, offset: number): void {
    const args = this.buffers.args.array as Uint32Array;
    const late = this.buffers.lateArgs.array as Uint32Array;

    late[at] = args[at];
    late[at + 1] = 0;
    late[at + 2] = args[at + 2];
    late[at + 3] = args[at + 3];
    late[at + 4] = args[at + 4] + offset;
  }

  private touch(slot: number): void {
    this.dirty.first = Math.min(this.dirty.first, slot);
    this.dirty.last = Math.max(this.dirty.last, slot);
    this.currentVersion += 1;
  }

  private static upload(attribute: BufferAttribute, first: number, last: number, stride: number): void {
    attribute.clearUpdateRanges();
    attribute.addUpdateRange(first * stride, (last - first + 1) * stride);
    attribute.needsUpdate = true;
  }
}
