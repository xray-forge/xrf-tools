import { describe, expect, it } from "@jest/globals";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { LevelObjectsDescription, LevelSpawnObjectDetails } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import {
  mockLevelObject,
  mockLevelVolumeObject,
  mockSelectedLevelDescription,
} from "@/core/level/fixtures/level.mocks";
import { toLevelObjectPick } from "@/core/level/lib/pick/level-pick";
import { ELevelSelectionMode } from "@/core/level/services/level-viewport";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { deferred, flush } from "@/fixtures/utils/promise";
import { LevelViewerObjectsService } from "@/tools/level-viewer/services/level-viewer-objects";

function mockDetails(id: number): LevelSpawnObjectDetails {
  return { customData: "", gameVertexId: 1, id, levelVertexId: 2 };
}

async function mockObjects(
  listed: LevelObjectsDescription = { objects: [], visuals: [] }
): Promise<ILevelViewContainerDescriptor & { objects: LevelViewerObjectsService }> {
  const level = mockSelectedLevelDescription();
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([LevelViewerObjectsService]);

  mockIpcCommands({
    [ELevelsCommand.DESCRIBE_OBJECTS]: mockSessionResponse(listed),
    [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(level),
  });
  await descriptor.load.load({
    source: { kind: "asset", logicalPath: "levels\\zaton" },
    roots: level.roots,
    isDltx: false,
    engine: EXrayEngineChoice.VANILLA,
  });

  return { ...descriptor, objects: descriptor.container.get(LevelViewerObjectsService) };
}

describe("LevelViewerObjectsService", () => {
  // Rows are walked through faster than the backend answers: what shows is the last one chosen.
  it("drops what was read of an object chosen before the last", async () => {
    const { objects } = await mockObjects();
    const first = deferred<SessionSnapshot<LevelSpawnObjectDetails>>();

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_SPAWN_OBJECT]: ({ index }) =>
        index === 0 ? first.promise : mockSessionSnapshot(mockDetails(401)),
    });

    const earlier: Promise<void> = objects.describe(0) as unknown as Promise<void>;

    await objects.describe(1);
    first.resolve(mockSessionSnapshot(mockDetails(400)));
    await earlier;

    expect(objects.details.value?.id).toBe(401);
  });

  it("forgets what was read once nothing is chosen", async () => {
    const { objects } = await mockObjects();

    mockIpcCommands({ [ELevelsCommand.DESCRIBE_SPAWN_OBJECT]: mockSessionResponse(mockDetails(400)) });

    await objects.describe(0);
    await objects.describe(null);

    expect(objects.details.state.isIdle).toBe(true);
    expect(objects.details.value).toBeNull();
  });

  it("isolates the selection by hiding every other object of the level", async () => {
    const kept = mockLevelVolumeObject(1);
    const others = [mockLevelObject(), mockLevelVolumeObject(2)];

    const { load, objects, viewport } = await mockObjects({ objects: [...others, kept], visuals: ["physics\\box"] });

    await flush();
    expect(load.objects?.objects).toHaveLength(3);

    viewport.notePicked(toLevelObjectPick(kept, null), ELevelSelectionMode.REPLACE);
    objects.isolateSelection();

    expect(viewport.hidden).toEqual(others);
  });
});
