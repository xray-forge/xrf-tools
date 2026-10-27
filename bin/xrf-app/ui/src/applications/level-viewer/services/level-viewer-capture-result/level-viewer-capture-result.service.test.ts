import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { save } from "@tauri-apps/plugin-dialog";
import { waitFor } from "@testing-library/react";
import { EventBus, WireEvent } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands } from "@xrf/ipc/testing";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ELevelAnnotationKind } from "@xrf/ipc/types/xrf-app-levels";

import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import { TLevelCapture } from "@/applications/level-viewer/lib/capture/level-captured";
import { DEFAULT_LEVEL_MAP_ANNOTATIONS } from "@/applications/level-viewer/lib/map/level-map-annotations";
import { DEFAULT_LEVEL_MAP_EXPORT } from "@/applications/level-viewer/lib/map/level-map-export";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/level-viewer-capture-result";
import { mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockApplicationScope } from "@/fixtures/utils/container";

const PATH: string = "appdata/level-viewer/screenshots/map_zaton-20261021-101500.png";

/** A map captured over Zaton's own map area. */
function mockMapCapture(url: string = "blob:map"): TLevelCapture {
  return {
    area: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
    image: { height: 2048, name: "map_zaton-20261021-101500.png", path: PATH, size: 4096, url, width: 2048 },
    mode: ELevelCaptureMode.MAP,
  };
}

function mockResult(raised: Array<INotificationPayload> = []): LevelViewerCaptureResultService {
  const { container } = mockLevelViewContainer([
    ...mockApplicationScope(EApplicationId.LEVEL_VIEWER),
    LevelViewerCaptureResultService,
  ]);

  jest.spyOn(LevelViewerCaptureResultService.prototype, "sessionId", "get").mockReturnValue("session");
  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  return container.get(LevelViewerCaptureResultService);
}

beforeEach(() => {
  URL.createObjectURL = jest.fn(() => "blob:annotations");
  URL.revokeObjectURL = jest.fn();
  mockIpcCommands({ [ELevelsCommand.WRITE_MAP_ANNOTATIONS]: mockSessionSnapshot("<svg/>", "session") });
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerCaptureResultService", () => {
  it("draws the map's annotations over the ground it covers, and again when what is drawn changes", async () => {
    const service: LevelViewerCaptureResultService = mockResult();
    const asked: Array<Record<string, unknown>> = [];

    mockIpcCommands({
      [ELevelsCommand.WRITE_MAP_ANNOTATIONS]: ({ request }) => {
        asked.push(request);

        return mockSessionSnapshot("<svg/>", "session");
      },
    });

    service.show(mockMapCapture());
    await service.drawAnnotations();

    expect(asked[0]).toEqual({
      path: PATH,
      area: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
      kinds: [],
      isLabelled: false,
    });
    expect(service.annotationsUrl).toBe("blob:annotations");

    service.setMapAnnotations({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: true });
    await service.drawAnnotations();

    expect(asked.at(-1)).toMatchObject({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: true });
  });

  it("writes the annotations in the order drawn and exports once the last is written, the capture held meanwhile", async () => {
    const service: LevelViewerCaptureResultService = mockResult();
    const events: Array<string> = [];
    const firstWrite: { finish: () => void } = { finish: () => undefined };

    jest.mocked(save).mockResolvedValueOnce("C:/out/map_zaton.dds");
    mockIpcCommands({
      [ELevelsCommand.WRITE_MAP_ANNOTATIONS]: ({ request }) => {
        events.push(`write ${request.kinds.length}`);

        return request.kinds.length
          ? mockSessionSnapshot("<svg/>", "session")
          : new Promise((resolve: (value: SessionSnapshot<string>) => void) => {
              firstWrite.finish = () => {
                events.push("written 0");
                resolve(mockSessionSnapshot("<svg/>", "session"));
              };
            });
      },
      [ELevelsCommand.EXPORT_MAP]: () => {
        events.push("export");

        return [2048, 2048];
      },
    });

    service.show(mockMapCapture());
    void service.drawAnnotations();
    service.setMapAnnotations({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: false });

    const exporting: Promise<void> = service.exportAs() as unknown as Promise<void>;

    await waitFor(() => expect(service.isExporting).toBe(true));
    service.back();

    expect(service.captured).not.toBeNull();

    firstWrite.finish();
    await exporting;

    expect(events).toEqual(["write 0", "written 0", "write 1", "export"]);
    expect(service.exported.value).toBe("C:/out/map_zaton.dds");
  });

  it("says why an action failed from the viewer, keeping the capture", async () => {
    const raised: Array<INotificationPayload> = [];
    const service: LevelViewerCaptureResultService = mockResult(raised);

    jest.spyOn(service.log, "error").mockImplementation(() => undefined);
    mockIpcCommands({
      [ELevelsCommand.DELETE_SCREENSHOT]: () => {
        throw new Error("the file is open elsewhere");
      },
    });

    service.show(mockMapCapture());
    await service.remove();

    expect(service.captured).not.toBeNull();
    expect(raised).toEqual([
      expect.objectContaining({
        details: `${PATH}\nthe file is open elsewhere`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.LEVEL_VIEWER,
        title: "Could not delete the capture",
      }),
    ]);
  });

  it("lets the picture it replaces go, with the export written of it", async () => {
    const service: LevelViewerCaptureResultService = mockResult();

    jest.mocked(save).mockResolvedValueOnce("C:/out/map_zaton.dds");
    mockIpcCommands({ [ELevelsCommand.EXPORT_MAP]: [2048, 2048] });

    service.show(mockMapCapture("blob:first"));
    await service.exportAs();
    expect(service.exported.value).toBe("C:/out/map_zaton.dds");

    service.show(mockMapCapture("blob:second"));

    expect(URL.revokeObjectURL).toHaveBeenCalledWith("blob:first");
    expect(service.exported.value).toBeNull();
  });

  it("puts every export and annotation setting back for a dialog opened afresh", () => {
    const service: LevelViewerCaptureResultService = mockResult();

    service.show(mockMapCapture());
    service.setMapExport({ longSide: 1024 });
    service.setMapAnnotations({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: true });
    service.reset();

    expect(service.captured).toBeNull();
    expect(service.mapExport).toEqual(DEFAULT_LEVEL_MAP_EXPORT);
    expect(service.mapAnnotations).toEqual(DEFAULT_LEVEL_MAP_ANNOTATIONS);
  });
});
