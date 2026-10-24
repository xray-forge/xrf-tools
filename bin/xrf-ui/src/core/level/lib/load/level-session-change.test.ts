import { describe, expect, it } from "@jest/globals";

import {
  ESessionRootChange,
  LEVEL_GEOMETRY_PART,
  LEVEL_MAP_PART,
  LEVEL_SURFACES_PART,
  LEVEL_TEXTURES_PART,
  SessionChange,
} from "@/core/ipc/types/xrf-app";

import { isLevelSceneStreamedAfresh } from "./level-session-change";

function kept(staleParts: Array<string>): SessionChange {
  return { revision: 2, root: { kind: ESessionRootChange.KEPT }, sessionId: "level", staleParts };
}

describe("level session changes", () => {
  it("streams the scene afresh only for what its sectors are packed and drawn from, and for a level derived anew", () => {
    expect(isLevelSceneStreamedAfresh(kept([LEVEL_GEOMETRY_PART]))).toBe(true);
    expect(isLevelSceneStreamedAfresh(kept([LEVEL_SURFACES_PART]))).toBe(true);
    expect(isLevelSceneStreamedAfresh(kept([LEVEL_MAP_PART]))).toBe(false);
    expect(isLevelSceneStreamedAfresh(kept([LEVEL_TEXTURES_PART]))).toBe(false);
    expect(
      isLevelSceneStreamedAfresh({
        revision: 2,
        root: { kind: ESessionRootChange.DERIVED },
        sessionId: "level",
        staleParts: [],
      })
    ).toBe(true);
    expect(
      isLevelSceneStreamedAfresh({
        revision: 1,
        root: { kind: ESessionRootChange.DERIVING },
        sessionId: "level",
        staleParts: [],
      })
    ).toBe(false);
  });
});
