import { SpawnSessionDescriptor } from "@/core/bindings/types/xrf-app";
import { AlifeObject, Patrol, SpawnFile, Vector3d } from "@/core/bindings/types/xrf-db";

/**
 * Creates a vector fixture.
 *
 * @param overrides - Field values to override.
 * @returns A vector fixture.
 */
export function mockVector3d(overrides: Partial<Vector3d> = {}): Vector3d {
  return { x: 0, y: 0, z: 0, ...overrides };
}

/**
 * Creates an ALife object fixture.
 *
 * @param overrides - Field values to override.
 * @returns An ALife object fixture.
 */
export function mockAlifeObject(overrides: Partial<AlifeObject> = {}): AlifeObject {
  return {
    clientDataSize: 0,
    clsid: "EStlk",
    direction: mockVector3d(),
    gameType: 1,
    id: 1,
    // The narrowest variant of the inherited union, since these fixtures only need a valid payload.
    inherited: {
      type: "CseAlifeGraphPoint",
      connectionPointName: "",
      connectionLevelName: "",
      location0: 0,
      location1: 0,
      location2: 0,
      location3: 0,
    },
    name: "esc_smart_stalker_1",
    netAction: 1,
    parentId: 65535,
    phantomId: 65535,
    position: mockVector3d({ x: 12.5, y: 1.25, z: -30 }),
    respawnTime: 0,
    scriptFlags: 0,
    scriptGameId: 0,
    scriptRp: 0,
    scriptVersion: 12,
    section: "stalker_novice",
    spawnId: 1,
    updateData: [],
    version: 124,
    ...overrides,
  };
}

/**
 * Creates a patrol fixture.
 *
 * @param overrides - Field values to override.
 * @returns A patrol fixture.
 */
export function mockPatrol(overrides: Partial<Patrol> = {}): Patrol {
  return {
    name: "esc_walker_walk",
    points: [
      {
        name: "wp00|a=patrol",
        position: mockVector3d({ x: 1, y: 0, z: 1 }),
        flags: 1,
        levelVertexId: 100,
        gameVertexId: 10,
      },
      {
        name: "wp01",
        position: mockVector3d({ x: 4, y: 0, z: 2 }),
        flags: 1,
        levelVertexId: 101,
        gameVertexId: 10,
      },
    ],
    links: [{ index: 0, links: [[1, 1]] }],
    ...overrides,
  };
}

/**
 * Creates a structurally complete spawn file fixture.
 *
 * @param overrides - Field values to override.
 * @returns A spawn file fixture.
 */
export function mockSpawnFile(overrides: Partial<SpawnFile> = {}): SpawnFile {
  return {
    header: {
      version: 124,
      guid: "8f1a1b2c-0000-4000-8000-000000000001",
      graphGuid: "8f1a1b2c-0000-4000-8000-000000000002",
      objectsCount: 2,
      levelsCount: 1,
    },
    alifeSpawn: {
      objects: [mockAlifeObject(), mockAlifeObject({ id: 2, name: "esc_smart_stalker_2", spawnId: 2 })],
    },
    artefactSpawn: {
      nodes: [{ position: mockVector3d({ x: 5, y: 0, z: 5 }), levelVertexId: 200, distance: 1.5 }],
    },
    patrols: { patrols: [mockPatrol()] },
    graphs: {
      header: {
        guid: "8f1a1b2c-0000-4000-8000-000000000002",
        edgesCount: 1,
        levelsCount: 1,
        pointsCount: 1,
        version: 10,
        verticesCount: 1,
      },
      levels: [
        {
          id: 1,
          guid: "8f1a1b2c-0000-4000-8000-000000000003",
          name: "l01_escape",
          offset: mockVector3d(),
          section: "level_escape",
        },
      ],
      crossTables: [
        {
          version: 16,
          gameGuid: "8f1a1b2c-0000-4000-8000-000000000002",
          levelGuid: "8f1a1b2c-0000-4000-8000-000000000003",
          nodesCount: 1,
          verticesCount: 1,
        },
      ],
      edges: [{ distance: 12.5, gameVertexId: 1 }],
      points: [{ distance: 3.25, levelVertexId: 100, position: mockVector3d({ x: 3, y: 0, z: 4 }) }],
      vertices: [
        {
          edgesCount: 1,
          edgesOffset: 0,
          levelId: 1,
          levelPointsCount: 1,
          levelPointsOffset: 0,
          levelVertexId: 100,
          vertexType: [0, 0, 0, 0],
          gamePoint: mockVector3d(),
          levelPoint: mockVector3d(),
        },
      ],
    },
    ...overrides,
  };
}

/** Creates a committed spawn-session fixture. */
export function mockSpawnSession(overrides: Partial<SpawnSessionDescriptor> = {}): SpawnSessionDescriptor {
  return {
    id: "8f1a1b2c-0000-4000-8000-000000000010",
    path: "C:\\game\\all.spawn",
    header: mockSpawnFile().header,
    ...overrides,
  };
}
