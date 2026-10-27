import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ELevelsBulkRoute } from "@xrf/ipc/commands/levels-bulk";
import { mockIpcBulkRoutes, mockIpcCommands } from "@xrf/ipc/testing";
import { LevelMapDescription, SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { RenderCubeCapture, RenderMapCapture } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import { ELevelMapArea, ELevelMapLook } from "@/applications/level-viewer/lib/map/level-map-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/capture-result";
import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/core/level/fixtures/level-container";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { clearLocalStorage } from "@/core/storage";
import { mockFlow } from "@/fixtures/utils/flow";

const PATH: string = "appdata/level-viewer/screenshots/map_zaton-20261021-101500.png";

const ZATON: SelectedLevelDescription = mockSelectedLevelDescription({
  bounds: {
    boundingBox: { min: { x: -700, y: -20, z: -700 }, max: { x: 700, y: 80, z: 700 } },
    boundingSphere: { center: { x: 0, y: 30, z: 0 }, radius: 990 },
  },
});

const ZATON_MAP: LevelMapDescription = {
  area: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
  texture: "map\\map_zaton",
};

function mockCapture(
  level: Nullable<SelectedLevelDescription> = ZATON,
  levelMap: Nullable<LevelMapDescription> = level ? ZATON_MAP : null
): ILevelViewContainerDescriptor & {
  service: LevelViewerCaptureService;
  result: LevelViewerCaptureResultService;
  raised: Array<INotificationPayload>;
} {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([
    LevelViewerCaptureResultService,
    LevelViewerCaptureService,
  ]);
  const raised: Array<INotificationPayload> = [];

  descriptor.container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );
  jest.spyOn(LevelViewerCaptureService.prototype, "level", "get").mockReturnValue(level);
  jest.spyOn(LevelViewerCaptureService.prototype, "levelMap", "get").mockReturnValue(levelMap);

  return {
    ...descriptor,
    raised,
    result: descriptor.container.get(LevelViewerCaptureResultService),
    service: descriptor.container.get(LevelViewerCaptureService),
  };
}

beforeEach(() => {
  clearLocalStorage();
  // A PNG's signature and header, 2048 by 2048.
  mockIpcBulkRoutes({
    [ELevelsBulkRoute.READ_SCREENSHOT]: Uint8Array.from([
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 8, 0, 0, 0, 8, 0,
    ]).buffer,
  });
  mockIpcCommands({ [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH });
  URL.createObjectURL = jest.fn(() => "blob:map");
  URL.revokeObjectURL = jest.fn();
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("LevelViewerCaptureService", () => {
  it("opens on the level's map, the whole box's heights, and the engine's look", () => {
    const { service } = mockCapture();

    service.open();

    expect(service.isOpen).toBe(true);
    expect(service.mapParameters).toMatchObject({
      area: ELevelMapArea.LEVEL_MAP,
      rect: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
      bottom: -20,
      top: 80,
      longSide: 2048,
      look: ELevelMapLook.MAP,
      isSpawned: false,
      supersampling: 2,
    });
    expect(service.mapSize).toEqual([2048, 2048]);
  });

  it("opens on the level's box, engine space, where it lays no map", () => {
    const { service } = mockCapture(ZATON, null);

    service.open();

    expect(service.mapParameters).toMatchObject({
      area: ELevelMapArea.LEVEL_GEOMETRY,
      rect: { minX: -700, minZ: -700, maxX: 700, maxZ: 700 },
    });
  });

  it("captures under the map weather with the spawned objects hidden, and puts the view back after", async () => {
    const { result, service, render, view, weather } = mockCapture();
    const capturingDuring: Array<boolean> = [];
    const cutTo = mockFlow(weather, "cutTo", async () => {
      capturingDuring.push(service.isCapturing);
    });
    const release = mockFlow(weather, "release");
    const shownDuring: Array<boolean> = [];
    const captured: Array<RenderMapCapture> = [];

    view.setOptions({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isSpawnedProps: true });
    jest.spyOn(render, "captureMap").mockImplementation(async (capture: RenderMapCapture) => {
      shownDuring.push(view.options.isSpawnedProps);
      captured.push(capture);

      return { outcome: "completed", width: 2048, height: 2048 };
    });

    service.open();
    await service.capture();

    expect(cutTo).toHaveBeenCalledWith("map");
    expect(capturingDuring).toEqual([true]);
    expect(shownDuring).toEqual([false]);
    expect(captured).toEqual([
      {
        minX: -600,
        minZ: -615,
        maxX: 600,
        maxZ: 585,
        bottom: -20,
        top: 80,
        width: 2048,
        height: 2048,
        supersampling: 2,
        settleFrames: 16,
        isGrassed: false,
      },
    ]);
    expect(view.options.isSpawnedProps).toBe(true);
    expect(release).toHaveBeenCalledTimes(1);
    expect(service.isCapturing).toBe(false);
    expect(result.captured).toMatchObject({
      mode: ELevelCaptureMode.MAP,
      image: { path: PATH, width: 2048, height: 2048 },
    });
  });

  it("captures at noon where the game has no map weather, then puts the clock back", async () => {
    const { raised, result, service, render, weather } = mockCapture();
    const timeDuring: Array<number> = [];

    mockFlow(weather, "cutTo", () => Promise.reject(new Error("no cycle 'map'")));
    mockFlow(weather, "release");
    jest.spyOn(render, "captureMap").mockImplementation(async () => {
      timeDuring.push(weather.time);

      return { outcome: "completed", width: 2048, height: 2048 };
    });

    weather.seekTo(3600);
    service.open();
    await service.capture();

    expect(timeDuring).toEqual([LEVEL_WEATHER_NOON]);
    expect(weather.time).toBe(3600);
    expect(raised.map((it: INotificationPayload) => it.severity)).toEqual([ENotificationSeverity.SUCCESS]);
    expect(result.captured?.mode).toBe(ELevelCaptureMode.MAP);
  });

  it("says why a capture failed, shows nothing, and puts the view back", async () => {
    const { raised, result, service, render, view, weather } = mockCapture();
    const release = mockFlow(weather, "release");

    view.setOptions({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isSpawnedProps: true });
    mockFlow(weather, "cutTo");
    jest.spyOn(render, "captureMap").mockRejectedValue(new Error("the device was lost"));

    service.open();
    await service.capture();

    expect(raised).toEqual([
      expect.objectContaining({
        details: expect.stringContaining("the device was lost"),
        severity: ENotificationSeverity.ERROR,
        title: "Could not capture the level map",
      }),
    ]);
    expect(result.captured).toBeNull();
    expect(service.isCapturing).toBe(false);
    expect(view.options.isSpawnedProps).toBe(true);
    expect(release).toHaveBeenCalledTimes(1);
  });

  it("keeps the parameters and shows no map where the capture was stopped", async () => {
    const { result, service, render, weather } = mockCapture();

    mockFlow(weather, "cutTo");
    mockFlow(weather, "release");
    jest.spyOn(render, "captureMap").mockResolvedValue({ outcome: "cancelled", width: 2048, height: 2048 });

    service.open();
    await service.capture();

    expect(result.captured).toBeNull();
    expect(service.mapParameters).not.toBeNull();
  });

  it("captures a cubemap around the camera under the view's own weather, its clock held", async () => {
    const { result, service, render, weather } = mockCapture();
    const cutTo = mockFlow(weather, "cutTo");
    const release = mockFlow(weather, "release");
    const pausedDuring: Array<boolean> = [];
    const captured: Array<RenderCubeCapture> = [];

    jest.spyOn(render, "captureCube").mockImplementation(async (capture: RenderCubeCapture) => {
      pausedDuring.push(weather.control.isPaused);
      captured.push(capture);

      return { outcome: "completed", width: 2048, height: 1536 };
    });

    service.openCubemap();
    service.setCubeParameters({ size: 1024 });
    await service.capture();

    expect(cutTo).not.toHaveBeenCalled();
    expect(pausedDuring).toEqual([true]);
    expect(captured).toEqual([{ size: 1024, supersampling: 2, settleFrames: 16 }]);
    expect(release).toHaveBeenCalledTimes(1);
    expect(result.captured?.mode).toBe(ELevelCaptureMode.CUBEMAP);
  });

  it("opens on a cubemap where the level covers no box to map", () => {
    const { service } = mockCapture({ ...ZATON, bounds: null });

    expect(service.canOpenMap).toBe(false);

    service.openCubemap();

    expect(service.isOpen).toBe(true);
    expect(service.mode).toBe(ELevelCaptureMode.CUBEMAP);
    expect(service.mapParameters).toBeNull();
  });

  it("draws the largest faces once over unless asked, and refuses a face no GPU draws", () => {
    const { service } = mockCapture();

    service.openCubemap();
    service.setCubeParameters({ size: 2048 });

    expect(service.cubeParameters.supersampling).toBe(1);
    expect(service.canCapture).toBe(true);

    service.setCubeParameters({ supersampling: 8 });

    expect(service.canCapture).toBe(false);
  });
});
