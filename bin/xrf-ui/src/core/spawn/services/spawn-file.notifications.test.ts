import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";

import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { SpawnFileService } from "@/core/spawn/services/spawn-file.service";
import { mockSpawnSession } from "@/fixtures/mocks/spawn.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";

interface IWatchedService {
  service: SpawnFileService;
  raised: Array<INotificationPayload>;
}

function watchNotifications(): IWatchedService {
  const { container, service }: IInjectedServiceMockDescriptor<SpawnFileService> =
    mockInjectedService(SpawnFileService);
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
    setMockInvokeResponses({});
  });

  it("reports a written save", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    setMockInvokeResponses({ "plugin:spawn|open_file": mockSpawnSession() });
    await service.openFile("source.spawn");

    await service.saveFile("C:\\out\\all.spawn");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].source).toBe("spawns");
    expect(raised[0].details).toContain("C:\\out\\all.spawn");
  });

  it("reports a save the backend refused", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    setMockInvokeResponses({ "plugin:spawn|open_file": mockSpawnSession() });
    await service.openFile("source.spawn");

    setMockInvokeResponses({
      ["plugin:spawn|save_file"]: () => {
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

    setMockInvokeResponses({ "plugin:spawn|open_file": mockSpawnSession() });
    await service.openFile("source.spawn");

    await service.saveUnpackedDirectory("C:\\out\\unpacked");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].details).toContain("C:\\out\\unpacked");
  });

  it("reports an export the backend refused", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    setMockInvokeResponses({ "plugin:spawn|open_file": mockSpawnSession() });
    await service.openFile("source.spawn");

    setMockInvokeResponses({
      ["plugin:spawn|save_unpacked_directory"]: () => {
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

    setMockInvokeResponses({
      ["plugin:spawn|open_file"]: () => {
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

    setMockInvokeResponses({
      ["plugin:spawn|get_session"]: () => {
        throw new Error("backend is gone");
      },
    });

    await service.onProvision(1);

    // Left unready, the editor parks on a spinner with no route back to the open form.
    expect(service.isReady).toBe(true);
    expect(service.isOpen).toBe(false);
  });

  it("says nothing about a file that opened", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    setMockInvokeResponses({
      ["plugin:spawn|open_file"]: mockSpawnSession(),
    });

    await service.openFile("C:\\game\\all.spawn");

    expect(service.isOpen).toBe(true);
    expect(service.path).toBe("C:\\game\\all.spawn");
    expect(raised).toHaveLength(0);
  });
});
