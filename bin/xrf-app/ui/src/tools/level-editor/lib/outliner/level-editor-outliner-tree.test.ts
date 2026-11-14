import { describe, expect, it } from "@jest/globals";
import {
  ESceneBinaryToolDescription,
  ESceneObjectProperties,
  ESceneProblemKind,
  ESceneToolFormat,
  ESceneToolKind,
  SceneProblem,
  SceneToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe } from "@xrf/types";
import { ITreeNode } from "@xrf/ui/tree";

import {
  mockSceneBinaryToolsDescription,
  mockSceneObject,
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockSceneToolDescription,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import {
  countLevelEditorOutlinerObjects,
  ELevelEditorOutlinerRow,
  ILevelEditorOutlinerSources,
  listLevelEditorOutlinerParentIds,
  TLevelEditorOutlinerRow,
  toLevelEditorOutlinerObjectId,
  toLevelEditorOutlinerTree,
} from "@/tools/level-editor/lib/outliner/level-editor-outliner-tree";

const UNREADABLE_WAYS: SceneProblem = {
  file: "way.part",
  kind: ESceneProblemKind.UNREADABLE,
  message: "Part's [main] does not read",
  tool: ESceneToolKind.WAY,
};

const CUSTOM_TOOL: SceneToolDescription = { files: [], format: null, kind: null, name: "custom_tool", size: 0 };

function mockSources(overrides: Partial<ILevelEditorOutlinerSources> = {}): ILevelEditorOutlinerSources {
  return {
    binaryTools: mockSceneBinaryToolsDescription(),
    objects: mockSceneObjectsDescription(),
    problems: [],
    tools: mockSelectedSceneDescription().tools,
    ...overrides,
  };
}

function toLabels(nodes: ReadonlyArray<ITreeNode<TLevelEditorOutlinerRow>> = []): Array<string> {
  return nodes.map((it: ITreeNode<TLevelEditorOutlinerRow>) => it.label);
}

function toCaptions(nodes: ReadonlyArray<ITreeNode<TLevelEditorOutlinerRow>>): Array<unknown> {
  return nodes.map((it: ITreeNode<TLevelEditorOutlinerRow>) =>
    it.payload?.kind === ELevelEditorOutlinerRow.TOOL ? it.payload.caption : null
  );
}

describe("toLevelEditorOutlinerTree", () => {
  it("lists every tool in the order the scene lists them, counting each, and an object tool's objects by name", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(mockSources(), { text: "" });

    expect(toLabels(tree)).toEqual(["Scene objects", "Spawn points", "AI map"]);
    expect(toCaptions(tree)).toEqual(["2", "3", "12,480 nodes"]);
    expect(toLabels(tree[0]?.children)).toEqual(["box_2", "box_10"]);
    expect(toLabels(tree[1]?.children)).toEqual(["medkit_1", "stalker_1", "stalker_2"]);
    expect(tree[2]?.children).toBeUndefined();
    expect(tree[1]?.children?.[0]?.id).toBe(toLevelEditorOutlinerObjectId({ index: 1, tool: ESceneToolKind.SPAWN }));
  });

  it("names a tool the editor does not know by its own name, where the scene lists it", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(
      mockSources({ tools: [...mockSelectedSceneDescription().tools, CUSTOM_TOOL] }),
      { text: "" }
    );

    expect(toLabels(tree)).toEqual(["Scene objects", "Spawn points", "AI map", "custom_tool"]);
  });

  it("lists a tool only a group's members or a problem name after the tools the scene lists", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(
      mockSources({
        objects: {
          problems: [],
          tools: [
            ...mockSceneObjectsDescription().tools,
            mockSceneObjectTool(ESceneToolKind.LIGHT, [
              mockSceneObject({ group: "lab_group", id: { index: 0, tool: ESceneToolKind.LIGHT }, name: "light_1" }),
            ]),
          ],
        },
        problems: [{ ...UNREADABLE_WAYS, tool: "custom_tool" }],
      }),
      { text: "" }
    );

    expect(toLabels(tree)).toEqual(["Scene objects", "Spawn points", "AI map", "Lights", "custom_tool"]);
  });

  // A tool whose objects did not read is still the scene's, and its row says why it lists nothing.
  it("marks a tool with its problems, listing it though nothing of it read", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(
      mockSources({
        problems: [UNREADABLE_WAYS, UNREADABLE_WAYS],
        tools: [mockSceneToolDescription(ESceneToolKind.WAY)],
      }),
      { text: "" }
    );
    const ways: Maybe<ITreeNode<TLevelEditorOutlinerRow>> = tree.find(
      (it: ITreeNode<TLevelEditorOutlinerRow>) => it.label === "Ways"
    );

    expect(ways?.payload).toEqual({
      caption: null,
      kind: ELevelEditorOutlinerRow.TOOL,
      problems: 2,
      tool: ESceneToolKind.WAY,
    });
    expect(ways?.children).toBeUndefined();
  });

  it("counts nothing for the tools whose parts are still to be read", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(
      mockSources({ binaryTools: null, objects: null }),
      { text: "" }
    );

    expect(toLabels(tree)).toEqual(["Scene objects", "Spawn points", "AI map"]);
    expect(toCaptions(tree)).toEqual([null, null, null]);
    expect(countLevelEditorOutlinerObjects(tree)).toBe(0);
  });

  it("keeps what a filter matches by name, reference or section, and only the tools holding it", () => {
    const sources: ILevelEditorOutlinerSources = mockSources({
      objects: {
        problems: [],
        tools: [
          ...mockSceneObjectsDescription().tools,
          mockSceneObjectTool(ESceneToolKind.PARTICLE, [
            mockSceneObject({
              id: { index: 0, tool: ESceneToolKind.PARTICLE },
              name: "fire_1",
              properties: { gameTypes: 1, kind: ESceneObjectProperties.PARTICLE, reference: "static\\fire" },
            }),
          ]),
        ],
      },
    });

    expect(toLabels(toLevelEditorOutlinerTree(sources, { text: "BOX_2" })[0]?.children)).toEqual(["box_2"]);
    expect(toLabels(toLevelEditorOutlinerTree(sources, { text: "props\\crate" })[0]?.children)).toEqual(["box_2"]);

    const stalkers: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(sources, { text: "stalker" });

    expect(toLabels(stalkers)).toEqual(["Spawn points"]);
    expect(toCaptions(stalkers)).toEqual(["2"]);
    expect(countLevelEditorOutlinerObjects(stalkers)).toBe(2);
    expect(toLabels(toLevelEditorOutlinerTree(sources, { text: "static\\fire" }))).toEqual(["Particles"]);
    expect(toLevelEditorOutlinerTree(sources, { text: "nothing" })).toEqual([]);
  });

  it("names the rows a filter opens, and counts the objects listed", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(mockSources(), { text: "" });

    expect(listLevelEditorOutlinerParentIds(tree)).toEqual(["tool:scene_object", "tool:spawn"]);
    expect(countLevelEditorOutlinerObjects(tree)).toBe(5);
  });

  it("takes a binary tool's counts by the name its part files carry", () => {
    const tree: Array<ITreeNode<TLevelEditorOutlinerRow>> = toLevelEditorOutlinerTree(
      mockSources({
        binaryTools: {
          problems: [],
          tools: [
            { marks: 1200, slots: 3, tool: ESceneBinaryToolDescription.WALLMARK },
            { objects: 4, slots: 2048, tool: ESceneBinaryToolDescription.DETAIL_OBJECT },
          ],
        },
        objects: null,
        tools: [
          mockSceneToolDescription(ESceneToolKind.DETAIL_OBJECT, ESceneToolFormat.BINARY),
          mockSceneToolDescription(ESceneToolKind.WALLMARK, ESceneToolFormat.BINARY),
        ],
      }),
      { text: "" }
    );

    expect(toLabels(tree)).toEqual(["Detail objects", "Wallmarks"]);
    expect(toCaptions(tree)).toEqual(["4 objects · 2,048 slots", "1,200 marks · 3 slots"]);
  });
});
