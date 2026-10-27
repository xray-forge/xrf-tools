import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { ESpawnCommand } from "@xrf/ipc/commands/spawn";
import { mockIpcCommands } from "@xrf/ipc/testing";

import { mockSpawnSession } from "@/applications/spawn-editor/fixtures/spawn.mocks";
import { SpawnFileService } from "@/applications/spawn-editor/services/spawn-file/spawn-file.service";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { IInjectedServiceMockDescriptor, mockApplicationScope, mockInjectedService } from "@/fixtures/utils/container";

interface IWatchedService {
  service: SpawnFileService;
  raised: Array<INotificationPayload>;
}

function watchNotifications(): IWatchedService {
  const { container, service }: IInjectedServiceMockDescriptor<SpawnFileService> = mockInjectedService(
    SpawnFileService,
    mockApplicationScope(EApplicationId.SPAWN_EDITOR)
  );
  const raised: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  return { raised, service };
}

describe("SpawnFileService notifications", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("reports a written save", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()) });
    await service.openFile("source.spawn");

    await service.saveFile("C:\\out\\all.spawn");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    // Every notice names the application it came from, not the group the spawn tools share.
    expect(raised[0].source).toBe(EApplicationId.SPAWN_EDITOR);
    expect(raised[0].details).toContain("C:\\out\\all.spawn");
  });

  it("reports a save the backend refused", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()) });
    await service.openFile("source.spawn");

    mockIpcCommands({
      [ESpawnCommand.SAVE_FILE]: () => {
        throw new Error("destination is read only");
      },
    });

    await service.saveFile("C:\\out\\all.spawn");

    // The failure is now carried by the operation rather than dropped, so the toolbar can report it.
    expect(service.operation.isLoading).toBe(false);
    expect(String(service.operation.error)).toContain("read only");
    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
  });

  it("reports a written export", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()) });
    await service.openFile("source.spawn");

    await service.saveUnpackedDirectory("C:\\out\\unpacked");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].details).toContain("C:\\out\\unpacked");
  });

  it("reports an export the backend refused", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()) });
    await service.openFile("source.spawn");

    mockIpcCommands({
      [ESpawnCommand.SAVE_UNPACKED_DIRECTORY]: () => {
        throw new Error("no such directory");
      },
    });

    await service.saveUnpackedDirectory("C:\\out\\unpacked");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].details).toContain("no such directory");
  });

  it("reports a spawn file that could not be opened", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({
      [ESpawnCommand.OPEN_FILE]: () => {
        throw new Error("not a spawn file");
      },
    });

    await service.openFile("C:\\game\\all.spawn");

    expect(service.isOpen).toBe(false);
    expect(service.path).toBeNull();
    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].details).toContain("not a spawn file");
  });

  it("becomes ready even when the session restore fails", async () => {
    const { service }: IWatchedService = watchNotifications();

    mockIpcCommands({
      [ESpawnCommand.RESTORE_FILE]: () => {
        throw new Error("backend is gone");
      },
    });

    await service.onProvision();

    // Left unready, the editor parks on a spinner with no route back to the open form.
    expect(service.isReady).toBe(true);
    expect(service.isOpen).toBe(false);
  });

  it("says nothing about a file that opened", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({
      [ESpawnCommand.OPEN_FILE]: mockSessionResponse(mockSpawnSession()),
    });

    await service.openFile("C:\\game\\all.spawn");

    expect(service.isOpen).toBe(true);
    expect(service.path).toBe("C:\\game\\all.spawn");
    expect(raised).toHaveLength(0);
  });
});
