import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { EExportsCommand } from "@xrf/ipc/commands/exports";
import { mockIpcCommands } from "@xrf/ipc/testing";

import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EToolId } from "@/core/routing/tool";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService, mockToolScope } from "@/fixtures/utils/container";
import { mockExportsProject } from "@/tools/exports-explorer/fixtures/exports.mocks";

import { ExportsService } from "./exports.service";

interface IWatchedService {
  service: ExportsService;
  raised: Array<INotificationPayload>;
}

function watchNotifications(): IWatchedService {
  const { container, service }: IInjectedServiceMockDescriptor<ExportsService> = mockInjectedService(
    ExportsService,
    mockToolScope(EToolId.EXPORTS_EXPLORER)
  );
  const raised: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  return { raised, service };
}

describe("ExportsService notifications", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("reports a project that could not be parsed", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({
      [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(() => {
        throw new Error("no scripts directory");
      }),
    });

    await service.openExportsProject("C:\\game\\scripts");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].source).toBe("exports-explorer");
    expect(raised[0].details).toContain("no scripts directory");
  });

  it("reports a refresh that could not complete", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(mockExportsProject()) });

    await service.openExportsProject("C:\\game\\scripts");

    mockIpcCommands({
      [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(() => {
        throw new Error("scripts moved");
      }),
    });

    await service.refreshExportsProject();

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].details).toContain("scripts moved");
  });

  it("reports the manifest it wrote", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(mockExportsProject()) });

    await service.openExportsProject("C:\\game\\scripts");
    await service.exportManifest("C:\\out\\extern.json");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].details).toBe("C:\\out\\extern.json");
  });

  it("reports a manifest that could not be written", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({
      [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(mockExportsProject()),
      [EExportsCommand.EXPORT_MANIFEST]: () => {
        throw new Error("destination is read only");
      },
    });

    await service.openExportsProject("C:\\game\\scripts");
    await service.exportManifest("C:\\out\\extern.json");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.ERROR);
    expect(raised[0].details).toContain("destination is read only");
    expect(service.manifest.error?.message).toContain("destination is read only");
  });

  it("says nothing about a project that opened", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    mockIpcCommands({ [EExportsCommand.OPEN_PROJECT]: mockSessionResponse(mockExportsProject()) });

    await service.openExportsProject("C:\\game\\scripts");

    expect(raised).toHaveLength(0);
  });
});
