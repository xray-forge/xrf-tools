import { describe, expect, it } from "@jest/globals";
import {
  ESceneToolKind,
  SCENE_HELPER_INDEX_BITS,
  SCENE_HELPER_TOOLS,
  SceneHelperToolEntry,
  SceneObjectId,
} from "@xrf/ipc/types/xrf-app-levels";

import { toSceneHelperId, toSceneHelperObject } from "@/tools/level-editor/lib/helpers/scene-helper-id";

/** The highest index an id holds. */
const MAX_INDEX: number = 2 ** SCENE_HELPER_INDEX_BITS - 1;

describe("scene helper ids", () => {
  // The backend makes the same id (`names_a_helper_by_its_tool_and_index_both_ways`): the way's place, 7, above 24 bits
  // of index.
  it("names the fifth way's helper as the backend does", () => {
    const way: SceneObjectId = { index: 5, tool: ESceneToolKind.WAY };

    expect(toSceneHelperId(way)).toBe(0x0700_0005);
    expect(toSceneHelperObject(0x0700_0005)).toEqual(way);
  });

  it("names every helped tool's objects both ways, to the last index an id holds", () => {
    for (const entry of SCENE_HELPER_TOOLS) {
      for (const index of [0, 1, MAX_INDEX]) {
        const object: SceneObjectId = { index, tool: entry.tool };
        const id: number = toSceneHelperId(object) as number;

        expect(id).toBeLessThan(2 ** 32);
        expect(toSceneHelperObject(id)).toEqual(object);
      }
    }
  });

  it("gives no id to a tool drawing no helper or an index past what an id holds", () => {
    expect(toSceneHelperId({ index: 0, tool: ESceneToolKind.SCENE_OBJECT })).toBeNull();
    expect(toSceneHelperId({ index: MAX_INDEX + 1, tool: ESceneToolKind.WAY })).toBeNull();
  });

  it("names no object by an id no helped tool's place answers", () => {
    const sceneObject: number = 2;

    expect(SCENE_HELPER_TOOLS.some((entry: SceneHelperToolEntry) => entry.kind === sceneObject)).toBe(false);
    expect(toSceneHelperObject(sceneObject * 2 ** SCENE_HELPER_INDEX_BITS)).toBeNull();
  });
});
