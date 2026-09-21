import { describe, expect, it } from "@jest/globals";

import { ILoadedSector, LevelSectorSet } from "@/core/level/lib/sector/level-sector-set";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { mockSectorDescription } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

function views(sector: number): ISectorViews {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer, { sector });

  return createSectorViews({ ...description, bufferLength: buffer.byteLength }, buffer.toArrayBuffer());
}

describe("LevelSectorSet", () => {
  it("builds the geometry of a sector it takes", () => {
    const set: LevelSectorSet = new LevelSectorSet();
    const loaded: ILoadedSector = set.adopt(views(0));

    expect(loaded.geometry.getAttribute("position").count).toBe(3);
    expect(set.has(0)).toBe(true);
    expect(set.size).toBe(1);
  });

  // The geometry is device memory, so dropping the reference is not enough.
  it("disposes the geometry of a sector it releases", () => {
    const set: LevelSectorSet = new LevelSectorSet();
    const loaded: ILoadedSector = set.adopt(views(0));

    let isDisposed: boolean = false;

    loaded.geometry.addEventListener("dispose", () => {
      isDisposed = true;
    });

    set.release(0);

    expect(isDisposed).toBe(true);
    expect(set.has(0)).toBe(false);
  });

  // Taking a sector already held replaces it, and the one replaced has to go with it.
  it("disposes what it replaces when a sector is taken twice", () => {
    const set: LevelSectorSet = new LevelSectorSet();
    const first: ILoadedSector = set.adopt(views(0));

    let isDisposed: boolean = false;

    first.geometry.addEventListener("dispose", () => {
      isDisposed = true;
    });

    set.adopt(views(0));

    expect(isDisposed).toBe(true);
    expect(set.size).toBe(1);
  });

  // What is held is the residency budget's question, and this is the only thing that can see a sector to answer
  // it. Everything reporting on it reads this rather than walking the sectors itself.
  it("says how much is held", () => {
    const set: LevelSectorSet = new LevelSectorSet();

    set.adopt(views(0));
    set.adopt(views(1));

    const measured = set.measure();

    expect(measured.sectors).toBe(2);
    expect(measured.bytes).toBe(Array.from(set.sizes().values()).reduce((total, it) => total + it, 0));
    expect(measured.bytes).toBeGreaterThan(0);
  });

  it("forgets a sector's size when it releases it", () => {
    const set: LevelSectorSet = new LevelSectorSet();

    set.adopt(views(0));
    set.release(0);

    expect(set.measure()).toEqual({ bytes: 0, sectors: 0 });
    expect(set.sizes().size).toBe(0);
  });

  it("releases everything it holds when the level goes", () => {
    const set: LevelSectorSet = new LevelSectorSet();

    set.adopt(views(0));
    set.adopt(views(1));
    set.dispose();

    expect(set.size).toBe(0);
    expect(set.measure().bytes).toBe(0);
  });
});
