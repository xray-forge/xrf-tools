import { SpawnBuildResult } from "@xrf/ipc/types/xrf-app-spawn";
import { Finding } from "@xrf/ipc/types/xrf-report";
import { SpawnListedLevel } from "@xrf/ipc/types/xrf-spawn-builder";

/**
 * A finished build of zaton and labx8 into `D:\qa\spawn\qa_levels.spawn`, its report's connections check holding
 * `findings`.
 *
 * @param findings - What the connections check found.
 * @returns The build's result.
 */
export function mockSpawnBuildResult(findings: Array<Finding> = []): SpawnBuildResult {
  return {
    outcome: "completed",
    spawn: {
      path: "D:\\qa\\spawn\\qa_levels.spawn",
      levels: ["zaton", "labx8"],
      guid: "00000000-0000-0000-0000-000000000001",
      graphGuid: "00000000-0000-0000-0000-000000000002",
      objects: 1234,
      artefactPoints: 56,
      patrols: 7,
      report: {
        status: "passed",
        checks: [
          { id: "spawn.graph-points", status: "passed", duration: null, findings: [] },
          { id: "spawn.connections", status: "passed", duration: null, findings },
        ],
      },
    },
  };
}

/**
 * The levels a game data lists, zaton and labx8 built and jupiter not.
 *
 * @returns The listing.
 */
export function mockSpawnListedLevels(): Array<SpawnListedLevel> {
  return [
    { name: "zaton", caption: "zaton", id: 108, isComplete: true },
    { name: "jupiter", caption: "jupiter", id: 109, isComplete: false },
    { name: "labx8", caption: "labx8", id: 150, isComplete: true },
  ];
}
