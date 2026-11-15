import { describe, expect, it } from "@jest/globals";
import {
  ESceneToolKind,
  SCENE_RENDER_INDEX_BITS,
  SCENE_RENDER_TOOLS,
  SceneObjectId,
  SceneToolKind,
} from "@xrf/ipc/types/xrf-app-levels";

import {
  toSceneRenderId,
  toSceneRenderIds,
  toSceneRenderObject,
} from "@/tools/level-editor/lib/render/scene-render-id";

/** The highest index an id holds. */
const MAX_INDEX: number = 2 ** SCENE_RENDER_INDEX_BITS - 1;

describe("scene render ids", () => {
  // The backend makes the same ids (`names_an_object_to_the_renderer_by_its_tool_and_index_both_ways`): the tool's
  // place, 7 for a way and 2 for a scene object, above 24 bits of index.
  it("names a way's helper and a scene object's model as the backend does", () => {
    const way: SceneObjectId = { index: 5, tool: ESceneToolKind.WAY };
    const box: SceneObjectId = { index: 3, tool: ESceneToolKind.SCENE_OBJECT };

    expect(toSceneRenderId(way)).toBe(0x0700_0005);
    expect(toSceneRenderObject(0x0700_0005)).toEqual(way);
    expect(toSceneRenderId(box)).toBe(0x0200_0003);
    expect(toSceneRenderObject(0x0200_0003)).toEqual(box);
  });

  it("names every tool's objects both ways, to the last index an id holds", () => {
    for (const tool of SCENE_RENDER_TOOLS) {
      for (const index of [0, 1, MAX_INDEX]) {
        const object: SceneObjectId = { index, tool };
        const id: number = toSceneRenderId(object) as number;

        expect(id).toBeLessThan(2 ** 32);
        expect(toSceneRenderObject(id)).toEqual(object);
      }
    }
  });

  it("gives no id to an index past what an id holds or a tool the editor does not know", () => {
    expect(toSceneRenderId({ index: MAX_INDEX + 1, tool: ESceneToolKind.WAY })).toBeNull();
    expect(toSceneRenderId({ index: 0, tool: "custom_tool" as SceneToolKind })).toBeNull();
  });

  it("names no object by an id no tool's place answers", () => {
    expect(toSceneRenderObject(SCENE_RENDER_TOOLS.length * 2 ** SCENE_RENDER_INDEX_BITS)).toBeNull();
    expect(toSceneRenderObject(-1)).toBeNull();
  });

  it("lists the ids of objects of any tool ascending", () => {
    const light: SceneObjectId = { index: 0, tool: ESceneToolKind.LIGHT };
    const stalker: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };
    const box: SceneObjectId = { index: 1, tool: ESceneToolKind.SCENE_OBJECT };

    // A scene object's tool stands before a light's, and a light's before a spawn point's.
    expect(toSceneRenderIds([stalker, box, light])).toEqual([
      toSceneRenderId(box),
      toSceneRenderId(light),
      toSceneRenderId(stalker),
    ]);
  });
});
