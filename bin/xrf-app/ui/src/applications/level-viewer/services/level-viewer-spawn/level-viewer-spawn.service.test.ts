import { describe, expect, it } from "@jest/globals";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { LevelSpawnObjectDetails } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { LevelViewerSpawnService } from "@/applications/level-viewer/services/level-viewer-spawn";
import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { deferred } from "@/fixtures/utils/promise";

function mockDetails(id: number): LevelSpawnObjectDetails {
  return { customData: "", gameVertexId: 1, id, levelVertexId: 2 };
}

async function mockSpawn(): Promise<ILevelViewContainerDescriptor & { spawn: LevelViewerSpawnService }> {
  const level = mockSelectedLevelDescription();
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([LevelViewerSpawnService]);

  mockIpcCommands({ [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(level) });
  await descriptor.load.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngineChoice.VANILLA,
  });

  return { ...descriptor, spawn: descriptor.container.get(LevelViewerSpawnService) };
}

describe("LevelViewerSpawnService", () => {
  // Rows are walked through faster than the backend answers: what shows is the last one chosen.
  it("drops what was read of an object chosen before the last", async () => {
    const { spawn } = await mockSpawn();
    const first = deferred<SessionSnapshot<LevelSpawnObjectDetails>>();

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_SPAWN_OBJECT]: ({ index }) =>
        index === 0 ? first.promise : mockSessionSnapshot(mockDetails(401)),
    });

    const earlier: Promise<void> = spawn.describe(0) as unknown as Promise<void>;

    await spawn.describe(1);
    first.resolve(mockSessionSnapshot(mockDetails(400)));
    await earlier;

    expect(spawn.details.value?.id).toBe(401);
  });

  it("forgets what was read once nothing is chosen", async () => {
    const { spawn } = await mockSpawn();

    mockIpcCommands({ [ELevelsCommand.DESCRIBE_SPAWN_OBJECT]: mockSessionResponse(mockDetails(400)) });

    await spawn.describe(0);
    await spawn.describe(null);

    expect(spawn.details.state.isIdle).toBe(true);
    expect(spawn.details.value).toBeNull();
  });
});
