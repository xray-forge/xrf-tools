import { describe, expect, it } from "@jest/globals";

import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { EStaticSlotKind, STATIC_NO_BATCH, STATIC_SLOT_WORDS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

function createPool(): { pool: StaticDrawPool; buffers: StaticDrawBuffers } {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers();
  const pool: StaticDrawPool = new StaticDrawPool(buffers);

  pool.isEnabled = true;

  return { buffers, pool };
}

function toRecord(buffers: StaticDrawBuffers, slot: number): Array<number> {
  return Array.from(
    (buffers.slots.array as Uint32Array).subarray(slot * STATIC_SLOT_WORDS, (slot + 1) * STATIC_SLOT_WORDS)
  );
}

describe("StaticDrawPool", () => {
  it("hands out no slot while static draws are off", () => {
    expect(new StaticDrawPool(new StaticDrawBuffers()).allocate()).toBeNull();
  });

  it("writes a draw's clusters, place, kind and batches into its slot's record", () => {
    const { buffers, pool } = createPool();
    const slot = pool.allocate() as number;

    pool.write(slot, EStaticSlotKind.SINGLE, { count: 3, start: 40 }, 9, 2, STATIC_NO_BATCH);

    expect(toRecord(buffers, slot)).toEqual([40, 3, 9, EStaticSlotKind.SINGLE, 2, STATIC_NO_BATCH, 0, 0]);
  });

  // The cull reads every slot up to the last handed out; a released one draws nothing and belongs to no batch.
  it("leaves a released slot drawing nothing and gives it to the next draw", () => {
    const { buffers, pool } = createPool();
    const first = pool.allocate() as number;

    pool.write(first, EStaticSlotKind.LISTED, { count: 2, start: 0 }, 0, 1, 1);
    pool.allocate();
    pool.release(first);

    expect(toRecord(buffers, first)).toEqual([0, 0, 0, EStaticSlotKind.NONE, STATIC_NO_BATCH, STATIC_NO_BATCH, 0, 0]);
    expect(pool.allocate()).toBe(first);
    expect(pool.count).toBe(2);
    expect(pool.extent).toBe(2);
  });

  it("uploads what changed as one span, and counts every change", () => {
    const { buffers, pool } = createPool();
    const version: number = pool.version;
    const slots: Array<number> = [pool.allocate(), pool.allocate(), pool.allocate()] as Array<number>;

    pool.write(slots[0], EStaticSlotKind.SINGLE, { count: 1, start: 0 }, 0, 0, 0);
    pool.write(slots[2], EStaticSlotKind.SINGLE, { count: 1, start: 1 }, 1, 0, 0);
    pool.flush();

    expect(pool.version).toBe(version + 2);
    expect(buffers.slots.updateRanges).toEqual([{ count: 3 * STATIC_SLOT_WORDS, start: 0 }]);
  });
});
