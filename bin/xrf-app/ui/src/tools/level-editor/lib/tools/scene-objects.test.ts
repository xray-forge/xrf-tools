import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind, SceneObjectDescription, SceneObjectToolDescription } from "@xrf/ipc/types/xrf-app-levels";

import { mockSceneObject, mockSceneObjectTool } from "@/tools/level-editor/fixtures/scene.mocks";
import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";

describe("findSceneObject", () => {
  const light: SceneObjectDescription = mockSceneObject({
    id: { index: 3, tool: ESceneToolKind.LIGHT },
    name: "light_0003",
  });
  const tools: Array<SceneObjectToolDescription> = [
    mockSceneObjectTool(ESceneToolKind.GLOW, []),
    mockSceneObjectTool(ESceneToolKind.LIGHT, [light]),
  ];

  it("finds an object by its tool and index", () => {
    expect(findSceneObject(tools, { index: 3, tool: ESceneToolKind.LIGHT })).toBe(light);
  });

  it("finds nothing for an index its tool does not hold, or a tool not listed", () => {
    expect(findSceneObject(tools, { index: 4, tool: ESceneToolKind.LIGHT })).toBeNull();
    expect(findSceneObject(tools, { index: 3, tool: ESceneToolKind.GLOW })).toBeNull();
    expect(findSceneObject(tools, { index: 3, tool: ESceneToolKind.WAY })).toBeNull();
  });
});
