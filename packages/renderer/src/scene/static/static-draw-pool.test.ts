import { describe, expect, it } from "@jest/globals";
import { Matrix4, Sphere, Vector3 } from "three/webgpu";

import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { EStaticPool, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

function createPool(): { pool: StaticDrawPool; buffers: StaticDrawBuffers } {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers();
  const pool: StaticDrawPool = new StaticDrawPool(buffers);

  pool.isEnabled = true;

  return { buffers, pool };
}

describe("StaticDrawPool", () => {
  it("hands out no slot while static draws are off", () => {
    expect(new StaticDrawPool(new StaticDrawBuffers()).allocate()).toBeNull();
  });

  it("writes a draw's arguments, sphere and matrix into its slot", () => {
    const { buffers, pool } = createPool();
    const slot = pool.allocate() as number;

    pool.write(slot, 30, 12, 7, new Sphere(new Vector3(1, 2, 3), 4), new Matrix4().makeTranslation(5, 0, 0));

    expect(Array.from((buffers.args.array as Uint32Array).subarray(0, 5))).toEqual([12, 1, 30, 7, slot]);
    expect(Array.from((buffers.spheres.array as Float32Array).subarray(0, 4))).toEqual([1, 2, 3, 4]);
    expect((buffers.models.array as Float32Array)[12]).toBe(5);
  });

  it("marks a slot drawing nothing so the cull keeps nothing of it", () => {
    const { buffers, pool } = createPool();
    const slot = pool.allocate() as number;

    pool.write(slot, 0, 0, 0, new Sphere(new Vector3(), 1), new Matrix4());

    expect((buffers.spheres.array as Float32Array)[3]).toBe(-1);
  });

  it("gives a released slot to the next draw", () => {
    const { pool } = createPool();
    const first = pool.allocate() as number;

    pool.allocate();
    pool.release(first);

    expect(pool.allocate()).toBe(first);
    expect(pool.count).toBe(2);
  });

  it("uploads what changed as one span a buffer, and counts every change", () => {
    const { buffers, pool } = createPool();
    const version: number = pool.version;
    const slots: Array<number> = [pool.allocate(), pool.allocate(), pool.allocate()] as Array<number>;

    pool.write(slots[0], 0, 3, 0, new Sphere(new Vector3(), 1), new Matrix4());
    pool.write(slots[2], 0, 3, 0, new Sphere(new Vector3(), 1), new Matrix4());
    pool.flush();

    expect(pool.version).toBe(version + 2);
    expect(buffers.args.updateRanges).toEqual([{ count: 15, start: 0 }]);
    expect(buffers.models.updateRanges).toEqual([{ count: 48, start: 0 }]);
  });

  it("gives every draw the same arguments for the second phase, with no instances until the second cull counts", () => {
    const { buffers, pool } = createPool();
    const slot = pool.allocate() as number;

    pool.write(slot, 30, 12, 7, new Sphere(new Vector3(), 1), new Matrix4());

    expect(Array.from((buffers.lateArgs.array as Uint32Array).subarray(0, 5))).toEqual([12, 0, 30, 7, slot]);
  });

  it("writes an instanced draw's arguments with no instances, its second list in the second half", () => {
    const { buffers, pool } = createPool();
    const slot = pool.allocate() as number;

    pool.writeListed(slot, 30, 12, 7, 100);

    expect(Array.from((buffers.args.array as Uint32Array).subarray(0, 5))).toEqual([12, 0, 30, 7, 100]);
    expect((buffers.lateArgs.array as Uint32Array)[4]).toBe(100 + buffers.capacity(EStaticPool.ROWS));
    // Culled by its rows, never as a slot.
    expect((buffers.spheres.array as Float32Array)[3]).toBe(-1);
  });

  it("hands out no slot past the buffers' capacity, and the ones they grow by once they have", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.SLOTS]: 2 });
    const pool: StaticDrawPool = new StaticDrawPool(buffers);

    pool.isEnabled = true;
    pool.allocate();
    pool.allocate();

    expect(pool.allocate()).toBeNull();

    buffers.grow(EStaticPool.SLOTS, 4);

    expect(pool.allocate()).toBe(2);
    expect(pool.capacity).toBe(4);
  });

  it("keeps a slot's arguments through a growth, and uploads them whole with the new buffers", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.SLOTS]: 2 });
    const pool: StaticDrawPool = new StaticDrawPool(buffers);

    pool.isEnabled = true;

    const slot = pool.allocate() as number;

    pool.write(slot, 30, 12, 7, new Sphere(new Vector3(1, 2, 3), 4), new Matrix4());
    buffers.grow(EStaticPool.SLOTS, 8);

    expect(Array.from((buffers.args.array as Uint32Array).subarray(0, 5))).toEqual([12, 1, 30, 7, slot]);
    expect(Array.from((buffers.spheres.array as Float32Array).subarray(0, 4))).toEqual([1, 2, 3, 4]);
  });

  it("moves every instanced draw's second list a row capacity on after the rows grow, and only theirs", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.ROWS]: 16 });
    const pool: StaticDrawPool = new StaticDrawPool(buffers);

    pool.isEnabled = true;

    const listed = pool.allocate() as number;
    const single = pool.allocate() as number;
    const late = buffers.lateArgs.array as Uint32Array;

    pool.writeListed(listed, 0, 3, 0, 5);
    pool.write(single, 0, 3, 0, new Sphere(new Vector3(), 1), new Matrix4());
    buffers.grow(EStaticPool.ROWS, 64);
    pool.relist();

    expect((buffers.lateArgs.array as Uint32Array)[listed * 5 + 4]).toBe(5 + 64);
    expect((buffers.lateArgs.array as Uint32Array)[single * 5 + 4]).toBe(single);
    expect(buffers.lateArgs.array).toBe(late);
  });

  it("keeps shadow instance lists disjoint through growth and clears every view when a slot is released", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.ROWS]: 16, [EStaticPool.SLOTS]: 4 });
    const pool: StaticDrawPool = new StaticDrawPool(buffers);

    pool.isEnabled = true;

    const slot = pool.allocate() as number;

    pool.writeListed(slot, 30, 12, 7, 5);
    expect(new Set(buffers.viewArgs.map((args) => args.array)).size).toBe(buffers.viewArgs.length);
    buffers.grow(EStaticPool.ROWS, 64);
    buffers.grow(EStaticPool.SLOTS, 8);
    pool.relist();

    buffers.viewArgs.forEach((args, view: number) => {
      expect(Array.from((args.array as Uint32Array).subarray(0, 5))).toEqual([12, 0, 30, 7, 5 + 64 * (2 + view)]);
      (args.array as Uint32Array)[1] = view + 1;
    });
    pool.release(slot);
    buffers.viewArgs.forEach((args) => expect((args.array as Uint32Array)[1]).toBe(0));
    expect(pool.allocate()).toBe(slot);
    pool.write(slot, 0, 3, 0, new Sphere(new Vector3(), 1), new Matrix4());
    buffers.viewArgs.forEach((args) => expect((args.array as Uint32Array)[4]).toBe(slot));
  });
});
