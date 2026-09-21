import { describe, expect, it } from "@jest/globals";

import { SectorOutline } from "@/core/ipc/types/xrf-visual";
import { mockSectorOutline } from "@/fixtures/mocks/level.mocks";

import {
  createLevelResidency,
  DEFAULT_LEVEL_RESIDENCY,
  ESTIMATED_SECTOR_BYTES,
  getSectorDistance,
  ILevelResidencyOptions,
  ILevelResidencyPlan,
  listLevelPreload,
  planLevelResidency,
} from "./level-residency";

const ORIGIN = { x: 0, y: 0, z: 0 };
const OPTIONS: ILevelResidencyOptions = {
  concurrency: 3,
  isPreloaded: true,
  keepDistance: 20,
  loadDistance: 10,
  maxSectors: 2,
  memoryBudget: Number.MAX_SAFE_INTEGER,
  minSectors: 0,
};

function getHolding(...sectors: Array<number>): ReadonlyMap<number, number> {
  return new Map(sectors.map((sector: number) => [sector, ESTIMATED_SECTOR_BYTES]));
}

/**
 * A sector sitting at one distance along x, with a unit sphere around it.
 */
function outlineAt(sector: number, x: number): SectorOutline {
  return mockSectorOutline({
    bounds: {
      boundingBox: { max: { x: x + 1, y: 1, z: 1 }, min: { x: x - 1, y: -1, z: -1 } },
      boundingSphere: { center: { x, y: 0, z: 0 }, radius: 1 },
    },
    sector,
  });
}

describe("level residency", () => {
  it("measures to the sector's surface rather than its centre", () => {
    expect(getSectorDistance(outlineAt(0, 10), ORIGIN)).toBe(9);
  });

  it("measures a camera inside a sector as being at it", () => {
    expect(getSectorDistance(outlineAt(0, 0.5), ORIGIN)).toBe(0);
  });

  // A sector reaching no drawable has nothing to draw, so it is never worth a read.
  it("never loads a sector that declares no extent", () => {
    const outlines: Array<SectorOutline> = [mockSectorOutline({ bounds: null, drawables: 0, sector: 0 })];

    expect(getSectorDistance(outlines[0] as SectorOutline, ORIGIN)).toBeNull();
    expect(planLevelResidency(outlines, ORIGIN, getHolding(), OPTIONS).load).toEqual([]);
  });

  it("loads the nearest sectors first", () => {
    const plan: ILevelResidencyPlan = planLevelResidency(
      [outlineAt(0, 9), outlineAt(1, 3), outlineAt(2, 6)],
      ORIGIN,
      getHolding(),
      OPTIONS
    );

    expect(plan.load).toEqual([1, 2]);
    expect(plan.resident).toEqual([1, 2]);
  });

  it("holds no more than the budget allows", () => {
    const plan: ILevelResidencyPlan = planLevelResidency(
      [outlineAt(0, 2), outlineAt(1, 3), outlineAt(2, 4), outlineAt(3, 5)],
      ORIGIN,
      getHolding(),
      OPTIONS
    );

    expect(plan.resident).toHaveLength(OPTIONS.maxSectors);
    expect(plan.resident).toEqual([0, 1]);
  });

  it("releases what the camera has left behind", () => {
    const plan: ILevelResidencyPlan = planLevelResidency([outlineAt(0, 100)], ORIGIN, getHolding(0), OPTIONS);

    expect(plan.evict).toEqual([0]);
    expect(plan.resident).toEqual([]);
  });

  // A camera sitting on the boundary would otherwise take and release the same sector every frame. The band is what
  // stops that: taken at the nearer distance, kept until the further one.
  it("keeps a held sector past the distance it would not be taken at", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 16)];

    expect(planLevelResidency(outlines, ORIGIN, getHolding(), OPTIONS).load).toEqual([]);
    expect(planLevelResidency(outlines, ORIGIN, getHolding(0), OPTIONS).resident).toEqual([0]);
    expect(planLevelResidency(outlines, ORIGIN, getHolding(0), OPTIONS).evict).toEqual([]);
  });

  // Two sectors the same distance away and only room for one: swapping them every frame would reload geometry for no
  // change on screen.
  it("prefers the sector already held when the budget has to choose", () => {
    const plan: ILevelResidencyPlan = planLevelResidency(
      [outlineAt(0, 5), outlineAt(1, 5), outlineAt(2, 5)],
      ORIGIN,
      getHolding(2),
      { ...OPTIONS, maxSectors: 1 }
    );

    expect(plan.resident).toEqual([2]);
    expect(plan.load).toEqual([]);
    expect(plan.evict).toEqual([]);
  });

  // A count says nothing about what a level costs. Twenty four sectors was throttling a level whose whole render
  // geometry is 150 MB, so a camera crossing it read the same sectors over and over.
  it("holds what the budget affords rather than a fixed number of sectors", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 1), outlineAt(1, 2), outlineAt(2, 3), outlineAt(3, 4)];
    const held: ReadonlyMap<number, number> = new Map([
      [0, 4],
      [1, 4],
      [2, 4],
      [3, 4],
    ]);

    const plan: ILevelResidencyPlan = planLevelResidency(outlines, ORIGIN, held, {
      ...OPTIONS,
      maxSectors: 4,
      memoryBudget: 10,
      minSectors: 0,
    });

    expect(plan.resident).toEqual([0, 1]);
    expect(plan.evict).toEqual([2, 3]);
  });

  // A budget too small for what the camera is standing in would draw nothing at all, which is worse than going
  // over it.
  it("keeps the nearest sectors even where the budget cannot pay for them", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 1), outlineAt(1, 2), outlineAt(2, 3)];

    const plan: ILevelResidencyPlan = planLevelResidency(outlines, ORIGIN, getHolding(), {
      ...OPTIONS,
      maxSectors: 3,
      memoryBudget: 1,
      minSectors: 2,
    });

    expect(plan.resident).toEqual([0, 1]);
  });

  // Nothing says what an unread sector costs, so the read ones say it. A level of large sectors holds fewer of
  // them than a level of small ones, without either being configured.
  it("estimates an unread sector from what the read ones came to", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 1), outlineAt(1, 2), outlineAt(2, 3)];

    const plan: ILevelResidencyPlan = planLevelResidency(outlines, ORIGIN, new Map([[0, 100]]), {
      ...OPTIONS,
      maxSectors: 3,
      memoryBudget: 250,
      minSectors: 0,
    });

    // One hundred held plus two estimated at a hundred each is three hundred, so the third does not fit.
    expect(plan.resident).toEqual([0, 1]);
  });

  it("asks for nothing when the camera is nowhere near the level", () => {
    const plan: ILevelResidencyPlan = planLevelResidency([outlineAt(0, 1000)], ORIGIN, getHolding(), OPTIONS);

    expect(plan).toEqual({ evict: [], load: [], resident: [] });
  });
});

describe("listLevelPreload", () => {
  it("orders the rest of the level by how near it is", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 30), outlineAt(1, 10), outlineAt(2, 20)];

    expect(listLevelPreload(outlines, ORIGIN, new Set())).toEqual([1, 2, 0]);
  });

  it("leaves out what is already held, which needs no reading", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 30), outlineAt(1, 10), outlineAt(2, 20)];

    expect(listLevelPreload(outlines, ORIGIN, getHolding(1))).toEqual([2, 0]);
  });

  it("leaves out a sector that reaches no drawable", () => {
    const outlines: Array<SectorOutline> = [outlineAt(0, 10), mockSectorOutline({ bounds: null, sector: 1 })];

    expect(listLevelPreload(outlines, ORIGIN, new Set())).toEqual([0]);
  });
});

describe("level residency against a real level", () => {
  it("loads something when the camera frames a level far larger than the default distance", () => {
    const radius: number = 1008;
    const framed = { x: 0, y: radius * 0.56, z: radius * 1.6 };
    const outlines: Array<SectorOutline> = [outlineAt(0, 0), outlineAt(1, 200), outlineAt(2, -200)];

    const fixed: ILevelResidencyPlan = planLevelResidency(outlines, framed, getHolding(), DEFAULT_LEVEL_RESIDENCY);
    const scaled: ILevelResidencyPlan = planLevelResidency(
      outlines,
      framed,
      getHolding(),
      createLevelResidency(radius)
    );

    expect(fixed.load.length).toBeGreaterThan(0);
    expect(scaled.load.length).toBeGreaterThan(0);
  });

  it("scales the distances to the level rather than holding them fixed", () => {
    const large = createLevelResidency(1008);

    expect(large.loadDistance).toBeGreaterThan(DEFAULT_LEVEL_RESIDENCY.loadDistance);
    expect(large.keepDistance).toBeGreaterThan(large.loadDistance);
  });

  // A level small enough that a fraction of it is nothing keeps the defaults, which are already generous for it.
  it("keeps the defaults as a floor for a small level", () => {
    expect(createLevelResidency(14)).toEqual(DEFAULT_LEVEL_RESIDENCY);
  });

  // The floor is what makes a blank screen impossible: whatever the distances say, the nearest sectors are held.
  it("holds the nearest sectors however far away the camera is", () => {
    const plan: ILevelResidencyPlan = planLevelResidency(
      [outlineAt(0, 100_000), outlineAt(1, 200_000)],
      ORIGIN,
      getHolding(),
      { ...OPTIONS, minSectors: 1 }
    );

    expect(plan.resident).toEqual([0]);
  });

  it("never holds more than the budget, whatever the floor asks for", () => {
    const plan: ILevelResidencyPlan = planLevelResidency(
      [outlineAt(0, 100_000), outlineAt(1, 200_000), outlineAt(2, 300_000)],
      ORIGIN,
      getHolding(),
      { ...OPTIONS, maxSectors: 2, minSectors: 9 }
    );

    expect(plan.resident).toEqual([0, 1]);
  });
});
