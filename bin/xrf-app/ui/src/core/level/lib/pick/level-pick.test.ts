import { describe, expect, it } from "@jest/globals";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { ERenderLevelHit, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { mockLevelSpawnObject } from "@/core/level/fixtures/level.mocks";
import { ELevelPick, relocateLevelPick, TLevelPick, toLevelPick } from "@/core/level/lib/pick/level-pick";
import { mockSurfaceDescriptor, mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";

describe("toLevelPick", () => {
  it("names a picked spawned object as the held spawn has it, and nothing for one not held", () => {
    const crate = mockLevelSpawnObject({ index: 5, name: "crate_5", visual: 1 });
    const spawn = { objects: [mockLevelSpawnObject(), crate], visuals: ["lamp", "crate"] };
    const hit: RenderLevelHit = { depth: 3, kind: ERenderLevelHit.SPAWN, object: 5, point: [1, 2, 3] };

    expect(toLevelPick(hit, spawn)).toEqual({
      kind: ELevelPick.SPAWN,
      object: crate,
      point: { x: 1, y: 2, z: -3 },
      visual: "crate",
    });
    expect(toLevelPick({ ...hit, object: 9 }, spawn)).toBeNull();
    expect(toLevelPick(hit, null)).toBeNull();
  });
});

describe("relocateLevelPick", () => {
  const wall = mockSurfaceDescriptor({ shader: "default", textures: ["wall"] });
  const floor = mockSurfaceDescriptor({ shader: "default", textures: ["floor"] });
  const surface: TLevelPick = {
    isImpostor: false,
    kind: ELevelPick.SURFACE,
    mesh: 2,
    place: 7,
    point: { x: 1, y: 2, z: 3 },
    sector: 4,
    shaderId: 0,
  };

  it("finds a spawned object again by its name, the first so named, where it stands now", () => {
    const crate = mockLevelSpawnObject({ index: 1, name: "crate" });
    const before = { objects: [mockLevelSpawnObject({ name: "lamp" }), crate], visuals: ["lamp"] };
    const moved = mockLevelSpawnObject({
      index: 0,
      name: "crate",
      transform: mockVisualTransform({ x: 5, y: 6, z: 7 }),
    });
    const after = { objects: [moved, { ...moved, index: 1 }], visuals: ["crate"] };
    const pick: TLevelPick = { kind: ELevelPick.SPAWN, object: crate, point: { x: 0, y: 0, z: 0 }, visual: "lamp" };

    expect(relocateLevelPick(pick, { spawn: before, surfaces: null }, { spawn: after, surfaces: null })).toEqual({
      kind: ELevelPick.SPAWN,
      object: moved,
      point: { x: 5, y: 6, z: -7 },
      visual: "crate",
    });
    expect(relocateLevelPick(pick, { spawn: before, surfaces: null }, { spawn: null, surfaces: null })).toBeNull();
  });

  it("finds a surface's entry again by its shader and textures, keeping the place it was picked at", () => {
    function relocate(after: Array<XraySurfaceDescriptor>): Nullable<TLevelPick> {
      return relocateLevelPick(surface, { spawn: null, surfaces: [wall, floor] }, { spawn: null, surfaces: after });
    }

    expect(relocate([floor, wall])).toEqual({ ...surface, shaderId: 1 });
    expect(relocate([wall])).toBe(surface);
    expect(relocate([floor, mockSurfaceDescriptor({ shader: "other", textures: ["wall"] })])).toBeNull();
  });

  it("keeps what was picked where its parts are the same, or the table is read for the first time", () => {
    const parts = { spawn: null, surfaces: [wall] };

    expect(relocateLevelPick(surface, parts, parts)).toBe(surface);
    expect(relocateLevelPick(surface, { spawn: null, surfaces: null }, { spawn: null, surfaces: [floor] })).toBe(
      surface
    );
  });
});
