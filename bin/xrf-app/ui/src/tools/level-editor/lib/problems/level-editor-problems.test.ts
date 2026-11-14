import { describe, expect, it } from "@jest/globals";
import { ESceneProblemKind, ESceneToolFormat, ESceneToolKind } from "@xrf/ipc/types/xrf-app-levels";

import { mockSceneToolDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import {
  describeSceneProblemKind,
  listLevelEditorProblems,
} from "@/tools/level-editor/lib/problems/level-editor-problems";

describe("listLevelEditorProblems", () => {
  it("lists what the open found, then what each part's read found, by kind, tool and file", () => {
    expect(
      listLevelEditorProblems({
        binaryTools: [
          {
            file: null,
            kind: ESceneProblemKind.UNREADABLE,
            message: "Tool's stream does not read",
            tool: ESceneToolKind.AI_MAP,
          },
        ],
        objects: [
          { file: null, kind: ESceneProblemKind.UNREADABLE, message: "Objects do not read", tool: ESceneToolKind.WAY },
        ],
        scene: [
          {
            file: "spawn_001.part",
            kind: ESceneProblemKind.FOREIGN_GUID,
            message: "Part carries another scene's GUID",
            tool: ESceneToolKind.SPAWN,
          },
          {
            file: "custom_tool.part",
            kind: ESceneProblemKind.UNREADABLE,
            message: "Part does not read",
            tool: "custom_tool",
          },
        ],
        tools: [
          mockSceneToolDescription(ESceneToolKind.SPAWN),
          mockSceneToolDescription(ESceneToolKind.WAY),
          mockSceneToolDescription(ESceneToolKind.AI_MAP, ESceneToolFormat.BINARY),
          { ...mockSceneToolDescription(ESceneToolKind.SPAWN), kind: null, name: "custom_tool" },
        ],
      })
    ).toEqual([
      {
        location: { line: null, path: "spawn_001.part" },
        message: "Part carries another scene's GUID",
        rule: "Another scene's part",
        subject: "Spawn points · spawn_001.part",
      },
      {
        location: { line: null, path: "custom_tool.part" },
        message: "Part does not read",
        rule: "Unreadable",
        subject: "custom_tool · custom_tool.part",
      },
      { message: "Objects do not read", rule: "Unreadable", subject: "Ways" },
      { message: "Tool's stream does not read", rule: "Unreadable", subject: "AI map" },
    ]);
  });

  it("lists nothing for a scene nothing is wrong with", () => {
    expect(listLevelEditorProblems({ binaryTools: [], objects: [], scene: [], tools: [] })).toEqual([]);
  });
});

describe("describeSceneProblemKind", () => {
  it("names every kind", () => {
    expect(Object.values(ESceneProblemKind).map(describeSceneProblemKind)).toEqual([
      "Unreadable",
      "Another scene's part",
      "Wrong tool version",
      "Another tool's class",
    ]);
  });
});
