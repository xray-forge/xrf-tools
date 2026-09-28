import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { StaticPlaces } from "#/scene/static/static-places";
import { StaticRows } from "#/scene/static/static-rows";
import { IStaticRunPool } from "#/scene/static/static-run-pool";
import { StaticSlotHolds } from "#/scene/static/static-slot-holds";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createHolds(): { holds: StaticSlotHolds; places: StaticPlaces; rows: StaticRows } {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), {
    [EStaticPool.PLACES]: 8,
    [EStaticPool.ROWS]: 8,
  });

  return {
    holds: new StaticSlotHolds((pool: IStaticRunPool, count: number): Nullable<number> => pool.allocate(count)),
    places: new StaticPlaces(buffers),
    rows: new StaticRows(buffers),
  };
}

describe("StaticSlotHolds", () => {
  it("keeps a slot's run while it wants one as long, and takes it again where it wants another length", () => {
    const { holds, rows } = createHolds();
    const free = jest.spyOn(rows, "free");

    expect(holds.hold(1, rows, 2)).toBe(0);
    expect(holds.hold(1, rows, 2)).toBe(0);
    expect(free).not.toHaveBeenCalled();
    expect(holds.hold(1, rows, 3)).toBe(0);
    expect(free).toHaveBeenCalledWith(0, 2);
    expect(rows.used).toBe(3);
  });

  it("holds nothing of a pool with no room, and frees what it held there", () => {
    const { holds, rows } = createHolds();

    holds.hold(1, rows, 2);

    expect(holds.hold(1, rows, 9)).toBeNull();
    expect(rows.used).toBe(0);
  });

  it("drops one pool's run, or every run a slot holds", () => {
    const { holds, places, rows } = createHolds();

    holds.hold(1, places, 1);
    holds.hold(1, rows, 4);
    holds.hold(2, rows, 2);
    holds.drop(1, places);

    expect([places.used, rows.used]).toEqual([0, 6]);

    holds.release(1);

    expect(rows.used).toBe(2);
    // Released, a slot holds again from nothing.
    expect(holds.hold(1, rows, 4)).toBe(0);
  });
});
