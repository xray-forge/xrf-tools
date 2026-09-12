import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";

import { ExportsService } from "@/applications/exports-explorer/services/exports/exports.service";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockExportsProject } from "@/fixtures/mocks/project.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";

interface IWatchedService {
  service: ExportsService;
  raised: Array<INotificationPayload>;
}

function watchNotifications(): IWatchedService {
  const { container, service }: IInjectedServiceMockDescriptor<ExportsService> = mockInjectedService(ExportsService);
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
    setMockInvokeResponses({});
  });

  it("reports a project that could not be parsed", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    setMockInvokeResponses({
      ["plugin:exports|open_project"]: mockSessionResponse(() => {
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

    setMockInvokeResponses({ ["plugin:exports|open_project"]: mockSessionResponse(mockExportsProject()) });

    await service.openExportsProject("C:\\game\\scripts");

    setMockInvokeResponses({
      ["plugin:exports|open_project"]: mockSessionResponse(() => {
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

    setMockInvokeResponses({ ["plugin:exports|open_project"]: mockSessionResponse(mockExportsProject()) });

    await service.openExportsProject("C:\\game\\scripts");
    await service.exportManifest("C:\\out\\extern.json");

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.SUCCESS);
    expect(raised[0].details).toBe("C:\\out\\extern.json");
  });

  it("reports a manifest that could not be written", async () => {
    const { raised, service }: IWatchedService = watchNotifications();

    setMockInvokeResponses({
      ["plugin:exports|open_project"]: mockSessionResponse(mockExportsProject()),
      ["plugin:exports|export_manifest"]: () => {
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

    setMockInvokeResponses({ ["plugin:exports|open_project"]: mockSessionResponse(mockExportsProject()) });

    await service.openExportsProject("C:\\game\\scripts");

    expect(raised).toHaveLength(0);
  });
});
