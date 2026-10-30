import { describe, expect, it } from "@jest/globals";
import { ELevelObjectId, LevelObject, LevelObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { ERenderLevelHit, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { mockLevelObject, mockLevelPatrol, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import {
  ELevelPick,
  relocateLevelObject,
  relocateLevelPick,
  TLevelPick,
  toLevelPick,
} from "@/core/level/lib/pick/level-pick";
import { mockSurfaceDescriptor, mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";

function mockSpawned(
  index: number,
  name: string,
  visual: number,
  transform = mockVisualTransform({ x: 0, y: 0, z: 0 })
): LevelObject {
  return mockLevelObject({ id: { index, source: ELevelObjectId.SPAWN }, name, transform }, { visual });
}

describe("toLevelPick", () => {
  it("names a picked spawned object as the held listing has it, and nothing for one not held", () => {
    const crate: LevelObject = mockSpawned(5, "crate_5", 1);
    const objects: LevelObjectsDescription = { objects: [mockLevelObject(), crate], visuals: ["lamp", "crate"] };
    const hit: RenderLevelHit = { depth: 3, kind: ERenderLevelHit.SPAWN, object: 5, point: [1, 2, 3] };

    expect(toLevelPick(hit, objects)).toEqual({
      kind: ELevelPick.OBJECT,
      object: crate,
      point: { x: 1, y: 2, z: -3 },
      visual: "crate",
    });
    expect(toLevelPick({ ...hit, object: 9 }, objects)).toBeNull();
    expect(toLevelPick(hit, null)).toBeNull();
  });

  // A helper is named by its object's place in the listing, whatever source the object comes from.
  it("names a picked helper by its object's place in the listing", () => {
    const objects: LevelObjectsDescription = {
      objects: [mockLevelObject(), ...mockLevelPatrol(0, "walk", 2)],
      visuals: [],
    };
    const hit: RenderLevelHit = { depth: 3, kind: ERenderLevelHit.HELPER, object: 3, point: [0, 0, 0] };

    expect(toLevelPick(hit, objects)).toMatchObject({ kind: ELevelPick.OBJECT, object: { name: "wp01" } });
    expect(toLevelPick({ ...hit, object: 9 }, objects)).toBeNull();
  });
});

describe("relocateLevelObject", () => {
  // Every path has its `wp00`: a point is found under its path, found again by name.
  it("finds a patrol point again under its path, though its name repeats across paths", () => {
    const before: LevelObjectsDescription = {
      objects: [...mockLevelPatrol(0, "walk", 2), ...mockLevelPatrol(1, "guard", 2)],
      visuals: [],
    };
    const after: LevelObjectsDescription = {
      objects: [...mockLevelPatrol(0, "guard", 3), ...mockLevelPatrol(1, "walk", 2)],
      visuals: [],
    };
    const point = before.objects[4] as LevelObject;

    expect(point.name).toBe("wp00");
    expect(relocateLevelObject(point, before, after)?.id).toEqual({ path: 0, point: 0, source: "patrolPoint" });
    expect(relocateLevelObject(before.objects[1] as LevelObject, before, after)?.id).toEqual({
      path: 1,
      point: 0,
      source: "patrolPoint",
    });
    expect(relocateLevelObject(point, before, { objects: [], visuals: [] })).toBeNull();
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

  it("finds an object again by its kind and name, the first so named, where it stands now", () => {
    const crate: LevelObject = mockSpawned(1, "crate", 0);
    const before: LevelObjectsDescription = { objects: [mockSpawned(0, "lamp", 0), crate], visuals: ["lamp"] };
    const moved: LevelObject = mockSpawned(0, "crate", 0, mockVisualTransform({ x: 5, y: 6, z: 7 }));
    // A volume of the same name is not the crate.
    const after: LevelObjectsDescription = {
      objects: [{ ...mockLevelVolumeObject(2), name: "crate" }, moved, mockSpawned(3, "crate", 0)],
      visuals: ["crate"],
    };
    const pick: TLevelPick = { kind: ELevelPick.OBJECT, object: crate, point: { x: 0, y: 0, z: 0 }, visual: "lamp" };

    expect(relocateLevelPick(pick, { objects: before, surfaces: null }, { objects: after, surfaces: null })).toEqual({
      kind: ELevelPick.OBJECT,
      object: moved,
      point: { x: 5, y: 6, z: -7 },
      visual: "crate",
    });
    expect(relocateLevelPick(pick, { objects: before, surfaces: null }, { objects: null, surfaces: null })).toBeNull();
  });

  it("finds a surface's entry again by its shader and textures, keeping the place it was picked at", () => {
    function relocate(after: Array<XraySurfaceDescriptor>): Nullable<TLevelPick> {
      return relocateLevelPick(surface, { objects: null, surfaces: [wall, floor] }, { objects: null, surfaces: after });
    }

    expect(relocate([floor, wall])).toEqual({ ...surface, shaderId: 1 });
    expect(relocate([wall])).toBe(surface);
    expect(relocate([floor, mockSurfaceDescriptor({ shader: "other", textures: ["wall"] })])).toBeNull();
  });

  it("keeps what was picked where its parts are the same, or the table is read for the first time", () => {
    const parts = { objects: null, surfaces: [wall] };

    expect(relocateLevelPick(surface, parts, parts)).toBe(surface);
    expect(relocateLevelPick(surface, { objects: null, surfaces: null }, { objects: null, surfaces: [floor] })).toBe(
      surface
    );
  });
});
