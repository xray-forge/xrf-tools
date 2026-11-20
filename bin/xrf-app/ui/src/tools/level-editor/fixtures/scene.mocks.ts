import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { IpcCommandMocks } from "@xrf/ipc/testing";
import {
  ESceneBinaryToolDescription,
  ESceneObjectProperties,
  ESceneSpawnKind,
  ESceneToolFormat,
  ESceneToolKind,
  SceneBinaryToolsDescription,
  SceneDrawDescription,
  SceneDrawProblem,
  SceneHelpersDescription,
  SceneObjectDescription,
  SceneObjectsDescription,
  SceneObjectToolDescription,
  SceneProblem,
  SceneSpawnField,
  SceneToolDescription,
  SceneToolFormat,
  SceneToolKind,
  SelectedSceneDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { createRoots } from "@/core/assets/lib/roots";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";

/** The GUID every part of the fixture scene carries. */
export const MOCK_SCENE_GUID: string = "1a2b3c4d:5e6f7a8b";

/**
 * A scene tool with one part file of `size` bytes, named after the tool and carrying the scene's GUID.
 */
export function mockSceneToolDescription(
  kind: SceneToolKind,
  format: SceneToolFormat = ESceneToolFormat.OBJECT,
  size: number = 2048
): SceneToolDescription {
  return { files: [{ guid: MOCK_SCENE_GUID, name: `${kind}.part`, size }], format, kind, name: kind, size };
}

/**
 * An open scene holding scene objects, spawn points and an AI map, with no problems.
 */
export function mockSelectedSceneDescription(
  overrides: Partial<SelectedSceneDescription> = {}
): SelectedSceneDescription {
  return {
    buildParams: {
      lightJitterSamples: 9,
      lightPixelPerMeter: 10,
      lightQuality: 2,
      smoothAngle: 75,
      weldDistance: 0.005,
    },
    camera: { position: { x: 1, y: 2, z: -5 }, rotation: { x: 0, y: Math.PI / 2, z: 0 } },
    game: { engine: EXrayEngineChoice.AUTO, isDltx: false, roots: createRoots([]) },
    guid: MOCK_SCENE_GUID,
    name: "labx8",
    objectsRoots: createRoots(["C:\\rawdata\\objects"]),
    problems: [],
    source: { path: "C:\\rawdata\\levels\\labx8.level" },
    tools: [
      mockSceneToolDescription(ESceneToolKind.SCENE_OBJECT),
      mockSceneToolDescription(ESceneToolKind.SPAWN),
      mockSceneToolDescription(ESceneToolKind.AI_MAP, ESceneToolFormat.BINARY, 65_536),
    ],
    ...overrides,
  };
}

/**
 * A library object placed at the origin, unless the overrides say otherwise.
 */
export function mockSceneObject(overrides: Partial<SceneObjectDescription> = {}): SceneObjectDescription {
  return {
    flags: 0,
    group: null,
    id: { index: 0, tool: ESceneToolKind.SCENE_OBJECT },
    name: "box_1",
    properties: {
      flags: 0,
      isFound: true,
      kind: ESceneObjectProperties.SCENE_OBJECT,
      reference: "props\\box",
    },
    transform: {
      position: { x: 0, y: 0, z: 0 },
      rotation: { x: 0, y: 0, z: 0 },
      scale: { x: 1, y: 1, z: 1 },
    },
    ...overrides,
  };
}

/**
 * A spawn point placing a server entity of `section`.
 *
 * @param index - Its place among the spawn tool's objects.
 * @param name - Its name.
 * @param section - The section its entity is created from.
 * @param data - Its packet's fields, as the scene saves them.
 */
export function mockSceneSpawn(
  index: number,
  name: string,
  section: string,
  data: Array<SceneSpawnField> = []
): SceneObjectDescription {
  return mockSceneObject({
    id: { index, tool: ESceneToolKind.SPAWN },
    name,
    properties: {
      attached: null,
      kind: ESceneObjectProperties.SPAWN,
      spawn: { data, flags: 0, kind: ESceneSpawnKind.ENTITY, section },
    },
  });
}

/**
 * One object tool of a scene, its objects in the order the editor loads them.
 */
export function mockSceneObjectTool(
  tool: SceneToolKind,
  objects: Array<SceneObjectDescription>
): SceneObjectToolDescription {
  return { objects, tool };
}

/**
 * The fixture scene's object tools: two library objects, one of them not found, and three spawn points.
 *
 * @param problems - The tools whose objects did not read.
 */
export function mockSceneObjectsDescription(problems: Array<SceneProblem> = []): SceneObjectsDescription {
  return {
    problems,
    tools: [
      mockSceneObjectTool(ESceneToolKind.SCENE_OBJECT, [
        mockSceneObject({ id: { index: 0, tool: ESceneToolKind.SCENE_OBJECT }, name: "box_10" }),
        mockSceneObject({
          id: { index: 1, tool: ESceneToolKind.SCENE_OBJECT },
          name: "box_2",
          properties: {
            flags: 0,
            isFound: false,
            kind: ESceneObjectProperties.SCENE_OBJECT,
            reference: "props\\crate",
          },
        }),
      ]),
      mockSceneObjectTool(ESceneToolKind.SPAWN, [
        mockSceneSpawn(0, "stalker_1", "stalker"),
        mockSceneSpawn(1, "medkit_1", "medkit"),
        mockSceneSpawn(2, "stalker_2", "stalker"),
      ]),
    ],
  };
}

/**
 * The fixture scene's binary tools: its AI map.
 *
 * @param problems - The binary tools whose parts did not read.
 */
export function mockSceneBinaryToolsDescription(problems: Array<SceneProblem> = []): SceneBinaryToolsDescription {
  return { problems, tools: [{ nodes: 12_480, tool: ESceneBinaryToolDescription.AI_MAP }] };
}

/**
 * The fixture scene's drawing: its library objects' extent, ten metres across about a point off the origin.
 *
 * @param problems - The scene objects it does not draw.
 */
export function mockSceneDrawDescription(problems: Array<SceneDrawProblem> = []): SceneDrawDescription {
  return {
    bounds: {
      boundingBox: { max: { x: 15, y: 5, z: -15 }, min: { x: 5, y: -5, z: -25 } },
      boundingSphere: { center: { x: 10, y: 0, z: -20 }, radius: 8.66 },
    },
    problems,
  };
}

/**
 * The fixture scene's helpers: its three spawn points' icons, about a point away from its drawn objects.
 */
export function mockSceneHelpersDescription(overrides: Partial<SceneHelpersDescription> = {}): SceneHelpersDescription {
  return {
    bounds: {
      boundingBox: { max: { x: -10, y: 2, z: 4 }, min: { x: -14, y: 0, z: 0 } },
      boundingSphere: { center: { x: -12, y: 1, z: 2 }, radius: 3 },
    },
    spawnModels: [],
    tools: [{ count: 3, tool: ESceneToolKind.SPAWN }],
    ...overrides,
  };
}

/** What a scene drawing no helper answers of its helpers. */
export const MOCK_NO_SCENE_HELPERS: SceneHelpersDescription = { bounds: null, spawnModels: [], tools: [] };

/** What a fixture scene's parts answer, each defaulting to the fixture's own. */
export interface IMockSceneParts {
  objects?: SceneObjectsDescription;
  binaryTools?: SceneBinaryToolsDescription;
  drawing?: SceneDrawDescription;
  helpers?: SceneHelpersDescription;
}

/**
 * Responses for the `describe_scene_*` queries, each a snapshot of the session asked about at `revision`.
 */
export function mockScenePartResponses(parts: IMockSceneParts = {}, revision: number = 0): IpcCommandMocks {
  return {
    [ELevelsCommand.DESCRIBE_SCENE_OBJECTS]: (args: { sessionId: string }) =>
      mockSessionSnapshot(parts.objects ?? mockSceneObjectsDescription(), args.sessionId, revision),
    [ELevelsCommand.DESCRIBE_SCENE_BINARY_TOOLS]: (args: { sessionId: string }) =>
      mockSessionSnapshot(parts.binaryTools ?? mockSceneBinaryToolsDescription(), args.sessionId, revision),
    [ELevelsCommand.DESCRIBE_SCENE_DRAW]: (args: { sessionId: string }) =>
      mockSessionSnapshot(parts.drawing ?? mockSceneDrawDescription(), args.sessionId, revision),
    [ELevelsCommand.DESCRIBE_SCENE_HELPERS]: (args: { sessionId: string }) =>
      mockSessionSnapshot(parts.helpers ?? mockSceneHelpersDescription(), args.sessionId, revision),
  };
}
