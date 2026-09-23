import { describe, expect, it } from "@jest/globals";
import { Matrix4, Sphere, Vector3 } from "three/webgpu";

import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { STATIC_ROW_CAPACITY, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

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
    expect((buffers.lateArgs.array as Uint32Array)[4]).toBe(100 + STATIC_ROW_CAPACITY);
    // Culled by its rows, never as a slot.
    expect((buffers.spheres.array as Float32Array)[3]).toBe(-1);
  });
});
