import { describe, expect, it } from "@jest/globals";
import { ERenderLevelHit, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";

import { mockLevelSpawnObject } from "@/core/level/fixtures/level.mocks";
import { ELevelPick, toLevelPick } from "@/core/level/lib/pick/level-pick";

describe("toLevelPick", () => {
  it("names a picked spawned object as the held spawn has it, and nothing for one not held", () => {
    const crate = mockLevelSpawnObject({ index: 5, name: "crate_5", visual: 1 });
    const spawn = { objects: [mockLevelSpawnObject(), crate], visuals: ["lamp", "crate"] };
    const hit: RenderLevelHit = { kind: ERenderLevelHit.SPAWN, object: 5, point: [1, 2, 3] };

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
