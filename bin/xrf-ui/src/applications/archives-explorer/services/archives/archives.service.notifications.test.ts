import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/archives.service";
import { ArchiveFileDescriptor } from "@/core/ipc/types/xrf-archive";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockArchiveFileDescriptor, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";

const FILE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "textures\\wpn.dds" });

interface IWatchedService {
  service: ArchivesService;
  raised: Array<INotificationPayload>;
}

function mockWatchedNotifications(): IWatchedService {
  const { container, service }: IInjectedServiceMockDescriptor<ArchivesService> = mockInjectedService(ArchivesService);

  service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes()));

  const raised: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  return { raised, service };
}

describe("ArchivesService notifications", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
  });

  it("reports a completed extraction, so the outcome survives leaving the editor", async () => {
    const { raised, service }: IWatchedService = mockWatchedNotifications();

    await service.extractFile(FILE, "C:\\out\\wpn.dds");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].source).toBe("archives-explorer");
    expect(raised[0].title).toContain("textures\\wpn.dds");
    expect(raised[0].details).toContain("C:\\out\\wpn.dds");
  });

  it("reports a refused extraction with what the backend said", async () => {
    const { raised, service }: IWatchedService = mockWatchedNotifications();

    setMockInvokeResponses({
      ["plugin:archives|extract_file"]: () => {
        throw new Error("destination is read only");
      },
    });

    await expect(service.extractFile(FILE, "C:\\out\\wpn.dds")).rejects.toThrow("read only");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].details).toContain("destination is read only");
  });

  it("counts what a directory extraction wrote", async () => {
    const { raised, service }: IWatchedService = mockWatchedNotifications();

    setMockInvokeResponses({
      ["plugin:archives|extract_directory"]: () => ({
        destination: "C:\\out",
        extractedCount: 12,
        prefix: "textures",
        size: 2048,
      }),
    });

    await service.extractArchiveDirectory("textures", "C:\\out");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].title).toContain("12 file(s)");
  });

  it("reports a project that could not be opened", async () => {
    const { raised, service }: IWatchedService = mockWatchedNotifications();

    setMockInvokeResponses({
      ["plugin:archives|open_volumes"]: mockSessionResponse(() => {
        throw new Error("not an archive directory");
      }),
    });

    await service.openVolumes("C:\\game");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].details).toContain("not an archive directory");
  });
});
