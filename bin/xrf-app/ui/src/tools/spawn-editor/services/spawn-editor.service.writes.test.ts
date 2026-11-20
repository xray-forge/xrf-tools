import { beforeEach, describe, expect, it } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { listMockInvokeCalls, mockIpcCommands } from "@xrf/ipc/testing";
import { ESpawnChunkKind, SpawnChunk } from "@xrf/ipc/types/xrf-app-spawn";

import { mockSessionRead, mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";
import { mockAlifeObject, mockSpawnFile, mockSpawnSession } from "@/tools/spawn-editor/fixtures/spawn.mocks";
import { SpawnEditorService } from "@/tools/spawn-editor/services/spawn-editor.service";

async function mockOpenedService(): Promise<SpawnEditorService> {
  const { service } = mockInjectedService(SpawnEditorService);

  mockIpcCommands({ [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()) });
  await service.open("C:\\game\\all.spawn");

  return service;
}

describe("SpawnEditorService writes", () => {
  beforeEach(() => mockIpcCommands({}));

  it("saves the open spawn to the path asked for, forced as asked", async () => {
    const service = await mockOpenedService();

    await service.saveFile("C:\\out\\all.spawn", true);
    await service.saveFile("C:\\out\\new.spawn", false);

    expect(listMockInvokeCalls(ESpawnCommand.SAVE_FILE)).toEqual([
      { path: "C:\\out\\all.spawn", sessionId: service.sessionId, isForced: true },
      { path: "C:\\out\\new.spawn", sessionId: service.sessionId, isForced: false },
    ]);
    expect(service.operation.value).toBe("save");
  });

  it("exports the open spawn into the directory asked for", async () => {
    const service = await mockOpenedService();

    await service.saveUnpackedDirectory("C:\\out\\unpacked", true);

    expect(listMockInvokeCalls(ESpawnCommand.SAVE_UNPACKED_DIRECTORY)).toEqual([
      { path: "C:\\out\\unpacked", sessionId: service.sessionId, isForced: true },
    ]);
    expect(service.operation.value).toBe("export");
  });

  it("writes nothing while no spawn is open", async () => {
    const { service } = mockInjectedService(SpawnEditorService);

    await service.saveFile("C:\\out\\all.spawn", true);
    await service.saveUnpackedDirectory("C:\\out\\unpacked", true);

    expect(listMockInvokeCalls(ESpawnCommand.SAVE_FILE)).toHaveLength(0);
    expect(listMockInvokeCalls(ESpawnCommand.SAVE_UNPACKED_DIRECTORY)).toHaveLength(0);
    expect(service.operation.value).toBeNull();
  });

  it("is busy while a write runs, so no second command races it", async () => {
    const service = await mockOpenedService();
    const answer = deferred<null>();

    mockIpcCommands({ [ESpawnCommand.SAVE_FILE]: () => answer.promise });

    const saving = flowResult(service.saveFile("C:\\out\\all.spawn", true));

    expect(service.isBusy).toBe(true);

    answer.resolve(null);
    await saving;

    expect(service.isBusy).toBe(false);
  });

  it("dismisses the last write's outcome on request", async () => {
    const service = await mockOpenedService();

    await service.saveFile("C:\\out\\all.spawn", true);
    service.clearOperation();

    expect(service.operation.value).toBeNull();
    expect(service.operation.error).toBeNull();
  });

  it("forgets the last write, every chunk read and the selected row once the file closes", async () => {
    const service = await mockOpenedService();

    mockIpcCommands({
      [ESpawnCommand.READ_CHUNK]: mockSessionRead<SpawnChunk>({
        kind: ESpawnChunkKind.ALIFE_SPAWN,
        ...mockSpawnFile().alifeSpawn,
      }),
    });
    await service.loadAlifeSpawn();
    await service.saveFile("C:\\out\\all.spawn", true);
    service.selectRow("Alife object", 1, mockAlifeObject());

    await service.close();

    expect(service.isOpen).toBe(false);
    expect(service.operation.value).toBeNull();
    expect(service.chunks.alifeSpawn.value).toBeNull();
    expect(service.selectedRow).toBeNull();
  });

  it("keeps the file, what was read of it and the last write when the backend refuses to close it", async () => {
    const service = await mockOpenedService();

    mockIpcCommands({
      [ESpawnCommand.READ_CHUNK]: mockSessionRead<SpawnChunk>({
        kind: ESpawnChunkKind.ALIFE_SPAWN,
        ...mockSpawnFile().alifeSpawn,
      }),
      [ESpawnCommand.CLOSE_FILE]: () => {
        throw new Error("backend is busy");
      },
    });
    await service.loadAlifeSpawn();
    await service.saveFile("C:\\out\\all.spawn", true);
    service.selectRow("Alife object", 1, mockAlifeObject());

    await service.close();

    expect(service.isOpen).toBe(true);
    expect(service.path).toBe("C:\\game\\all.spawn");
    expect(service.operation.value).toBe("save");
    expect(service.chunks.alifeSpawn.value).not.toBeNull();
    expect(service.selectedRow?.id).toBe(1);
  });
});
