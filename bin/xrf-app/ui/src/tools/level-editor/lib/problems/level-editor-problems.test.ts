import { describe, expect, it } from "@jest/globals";
import {
  ESceneDrawProblemKind,
  ESceneObjectProperties,
  ESceneProblemKind,
  ESceneToolFormat,
  ESceneToolKind,
  SceneObjectToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { ERenderTextureGroup } from "@xrf/ipc/types/xrf-renderer";
import { IEditorProblem } from "@xrf/ui/editor/problems";

import { ELevelTextureProblemRule } from "@/core/level/lib/problems/level-texture-problem-rule";
import {
  mockSceneObject,
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockSceneToolDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import {
  describeSceneDrawProblemKind,
  describeSceneProblemKind,
  ILevelEditorProblemSources,
  LEVEL_EDITOR_MODEL_PROBLEM_RULE,
  LEVEL_EDITOR_SPAWN_MODEL_PROBLEM_RULE,
  listLevelEditorProblems,
} from "@/tools/level-editor/lib/problems/level-editor-problems";

/** Nothing wrong with a scene, each source empty. */
const NO_PROBLEMS: ILevelEditorProblemSources = {
  binaryTools: [],
  drawing: [],
  models: [],
  objectTools: [],
  objects: [],
  scene: [],
  spawnModels: [],
  textures: [],
  tools: [],
};

describe("listLevelEditorProblems", () => {
  it("lists what the open found, then what each part's read found, by kind, tool and file", () => {
    expect(
      listLevelEditorProblems({
        ...NO_PROBLEMS,
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

  it("lists the scene objects the picture leaves out after the parts, by name and library object", () => {
    expect(
      listLevelEditorProblems({
        ...NO_PROBLEMS,
        drawing: [
          {
            kind: ESceneDrawProblemKind.MISSING,
            message: "No library object 'props\\crate' in the objects roots",
            object: { index: 1, tool: ESceneToolKind.SCENE_OBJECT },
            reference: "props\\crate",
          },
          {
            kind: ESceneDrawProblemKind.UNREADABLE,
            message: "Library object 'props\\barrel' does not read",
            object: { index: 7, tool: ESceneToolKind.SCENE_OBJECT },
            reference: "props\\barrel",
          },
        ],
        objectTools: mockSceneObjectsDescription().tools,
        objects: [
          { file: null, kind: ESceneProblemKind.UNREADABLE, message: "Objects do not read", tool: ESceneToolKind.WAY },
        ],
      })
    ).toEqual([
      { message: "Objects do not read", rule: "Unreadable", subject: "way" },
      {
        message: "No library object 'props\\crate' in the objects roots",
        rule: "Library object missing",
        subject: "box_2 · props\\crate",
      },
      // An object the listing does not hold is named by its tool and its place among the tool's objects.
      {
        message: "Library object 'props\\barrel' does not read",
        rule: "Library object unreadable",
        subject: "Scene objects #7 · props\\barrel",
      },
    ]);
  });

  // The renderer's textures are listed as the level viewer lists them, by the same lister.
  it("lists the textures the renderer could not answer for after the drawing, under the viewer's rules", () => {
    expect(
      listLevelEditorProblems({
        ...NO_PROBLEMS,
        drawing: [
          {
            kind: ESceneDrawProblemKind.MISSING,
            message: "No library object 'props\\crate' in the objects roots",
            object: { index: 1, tool: ESceneToolKind.SCENE_OBJECT },
            reference: "props\\crate",
          },
        ],
        objectTools: mockSceneObjectsDescription().tools,
        textures: [
          { group: ERenderTextureGroup.SURFACES, reason: "missing", reference: "terrain\\terrain_zaton" },
          { group: ERenderTextureGroup.GRASS, reason: "missing", reference: "grass\\grass_1" },
        ],
      }).map((problem: IEditorProblem) => [problem.rule, problem.subject])
    ).toEqual([
      ["Library object missing", "box_2 · props\\crate"],
      [ELevelTextureProblemRule.TEXTURE, "terrain\\terrain_zaton"],
      [ELevelTextureProblemRule.GRASS_TEXTURE, "grass\\grass_1"],
    ]);
  });

  it("lists each library object the renderer could not model last, by the scene objects placing it", () => {
    const objectTools: Array<SceneObjectToolDescription> = [
      mockSceneObjectTool(ESceneToolKind.SCENE_OBJECT, [
        mockSceneObject({ id: { index: 0, tool: ESceneToolKind.SCENE_OBJECT }, name: "barrel_1" }),
        mockSceneObject({ id: { index: 1, tool: ESceneToolKind.SCENE_OBJECT }, name: "barrel_2" }),
        mockSceneObject({ id: { index: 2, tool: ESceneToolKind.SCENE_OBJECT }, name: "barrel_3" }),
        mockSceneObject({
          id: { index: 3, tool: ESceneToolKind.SCENE_OBJECT },
          name: "shell_1",
          properties: {
            flags: 0,
            isFound: true,
            kind: ESceneObjectProperties.SCENE_OBJECT,
            reference: "levels\\shell",
          },
        }),
      ]),
    ];

    expect(
      listLevelEditorProblems({
        ...NO_PROBLEMS,
        models: [
          { name: "props\\box", reason: "Library object 'props\\box' does not compile: no surfaces" },
          { name: "levels\\shell", reason: "Library object 'levels\\shell' does not compile: no meshes" },
          { name: "props\\gone", reason: "No scene object places library object 'props\\gone'" },
        ],
        objectTools,
        textures: [{ group: ERenderTextureGroup.SURFACES, reason: "missing", reference: "wood\\planks" }],
      })
    ).toEqual([
      { message: "missing", rule: ELevelTextureProblemRule.TEXTURE, subject: "wood\\planks" },
      {
        message:
          "barrel_1 and 2 more scene objects are not drawn: Library object 'props\\box' does not compile: no surfaces",
        rule: LEVEL_EDITOR_MODEL_PROBLEM_RULE,
        subject: "barrel_1 · props\\box",
      },
      {
        message: "shell_1 is not drawn: Library object 'levels\\shell' does not compile: no meshes",
        rule: LEVEL_EDITOR_MODEL_PROBLEM_RULE,
        subject: "shell_1 · levels\\shell",
      },
      {
        message: "Its scene objects are not drawn: No scene object places library object 'props\\gone'",
        rule: LEVEL_EDITOR_MODEL_PROBLEM_RULE,
        subject: "props\\gone",
      },
    ]);
  });

  it("lists each spawn point model the renderer could not read by the spawn points standing as it", () => {
    const objectTools: Array<SceneObjectToolDescription> = [
      mockSceneObjectTool(ESceneToolKind.SPAWN, [
        mockSceneObject({ id: { index: 0, tool: ESceneToolKind.SPAWN }, name: "box_1" }),
        mockSceneObject({ id: { index: 1, tool: ESceneToolKind.SPAWN }, name: "box_2" }),
        mockSceneObject({ id: { index: 2, tool: ESceneToolKind.SPAWN }, name: "lamp_1" }),
      ]),
    ];
    const boxModel: string = "dynamics\\box\\box_wood.ogf";
    const lampModel: string = "dynamics\\light\\lamp.ogf";

    expect(
      listLevelEditorProblems({
        ...NO_PROBLEMS,
        models: [
          { name: boxModel, reason: "Failed to read visual 'dynamics\\box\\box_wood': not a model" },
          { name: lampModel, reason: "Failed to read visual 'dynamics\\light\\lamp': not a model" },
          { name: "props\\box", reason: "Library object 'props\\box' does not compile: no surfaces" },
        ],
        objectTools,
        spawnModels: [
          { model: boxModel, object: { index: 0, tool: ESceneToolKind.SPAWN } },
          { model: boxModel, object: { index: 1, tool: ESceneToolKind.SPAWN } },
          { model: lampModel, object: { index: 2, tool: ESceneToolKind.SPAWN } },
        ],
      })
    ).toEqual([
      {
        message:
          "box_1 and 1 more spawn point are not drawn as a model: Failed to read visual 'dynamics\\box\\box_wood': " +
          "not a model",
        rule: LEVEL_EDITOR_SPAWN_MODEL_PROBLEM_RULE,
        subject: `box_1 · ${boxModel}`,
      },
      {
        message: "lamp_1 is not drawn as a model: Failed to read visual 'dynamics\\light\\lamp': not a model",
        rule: LEVEL_EDITOR_SPAWN_MODEL_PROBLEM_RULE,
        subject: `lamp_1 · ${lampModel}`,
      },
      {
        message: "Its scene objects are not drawn: Library object 'props\\box' does not compile: no surfaces",
        rule: LEVEL_EDITOR_MODEL_PROBLEM_RULE,
        subject: "props\\box",
      },
    ]);
  });

  // An unreadable `.object` is in the drawing already, by scene object; the renderer meets it again reading it.
  it("lists no library object the drawing found unreadable again among the models", () => {
    expect(
      listLevelEditorProblems({
        ...NO_PROBLEMS,
        drawing: [
          {
            kind: ESceneDrawProblemKind.UNREADABLE,
            message: "Library object 'props\\box' does not read",
            object: { index: 0, tool: ESceneToolKind.SCENE_OBJECT },
            reference: "props\\box",
          },
        ],
        models: [{ name: "props\\box", reason: "Library object 'props\\box' does not read" }],
        objectTools: mockSceneObjectsDescription().tools,
      }).map((problem: IEditorProblem) => problem.rule)
    ).toEqual(["Library object unreadable"]);
  });

  it("lists nothing for a scene nothing is wrong with", () => {
    expect(listLevelEditorProblems(NO_PROBLEMS)).toEqual([]);
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

describe("describeSceneDrawProblemKind", () => {
  it("names every kind", () => {
    expect(Object.values(ESceneDrawProblemKind).map(describeSceneDrawProblemKind)).toEqual([
      "Library object missing",
      "Library object unreadable",
    ]);
  });
});
