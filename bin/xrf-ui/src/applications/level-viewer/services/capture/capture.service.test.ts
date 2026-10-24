import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { save } from "@tauri-apps/plugin-dialog";
import { waitFor } from "@testing-library/react";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ELevelsBulkRoute } from "@xrf/ipc/commands/levels-bulk";
import { mockIpcBulkRoutes, mockIpcCommands } from "@xrf/ipc/testing";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { ELevelAnnotationKind, LevelMapDescription, SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { RenderCubeCapture, RenderMapCapture } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import { ELevelMapArea, ELevelMapLook } from "@/applications/level-viewer/lib/map/level-map-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/capture";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { clearLocalStorage } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { ILevelViewContainerDescriptor, mockLevelViewContainer } from "@/fixtures/utils/level-container";

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
): ILevelViewContainerDescriptor & { service: LevelViewerCaptureService } {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer([LevelViewerCaptureService]);

  jest.spyOn(LevelViewerCaptureService.prototype, "level", "get").mockReturnValue(level);
  jest.spyOn(LevelViewerCaptureService.prototype, "levelMap", "get").mockReturnValue(levelMap);

  return { ...descriptor, service: descriptor.container.get(LevelViewerCaptureService) };
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
    const { service, render, view, weather } = mockCapture();
    const capturingDuring: Array<boolean> = [];
    const cutTo = jest.spyOn(weather, "cutTo").mockImplementation(async () => {
      capturingDuring.push(service.isCapturing);
    });
    const release = jest.spyOn(weather, "release").mockResolvedValue();
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
    expect(service.captured).toMatchObject({
      mode: ELevelCaptureMode.MAP,
      image: { path: PATH, width: 2048, height: 2048 },
    });
  });

  it("captures at noon where the game has no map weather, then puts the clock back", async () => {
    const { service, render, weather } = mockCapture();
    const timeDuring: Array<number> = [];

    jest.spyOn(weather, "cutTo").mockRejectedValue(new Error("no cycle 'map'"));
    jest.spyOn(weather, "release").mockResolvedValue();
    jest.spyOn(render, "captureMap").mockImplementation(async () => {
      timeDuring.push(weather.time);

      return { outcome: "completed", width: 2048, height: 2048 };
    });

    weather.seekTo(3600);
    service.open();
    await service.capture();

    expect(timeDuring).toEqual([LEVEL_WEATHER_NOON]);
    expect(weather.time).toBe(3600);
    expect(service.failure).toBeNull();
    expect(service.captured?.mode).toBe(ELevelCaptureMode.MAP);
  });

  it("says why a capture failed, shows nothing, and puts the view back", async () => {
    const { service, render, view, weather } = mockCapture();
    const release = jest.spyOn(weather, "release").mockResolvedValue();

    view.setOptions({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isSpawnedProps: true });
    jest.spyOn(weather, "cutTo").mockResolvedValue();
    jest.spyOn(render, "captureMap").mockRejectedValue(new Error("the device was lost"));

    service.open();
    await service.capture();

    expect(service.failure).toContain("the device was lost");
    expect(service.captured).toBeNull();
    expect(service.isCapturing).toBe(false);
    expect(view.options.isSpawnedProps).toBe(true);
    expect(release).toHaveBeenCalledTimes(1);
  });

  it("draws the map's annotations over the ground it covers, and again when what is drawn changes", async () => {
    jest.spyOn(LevelViewerCaptureService.prototype, "sessionId", "get").mockReturnValue("session");

    const { service, render, weather } = mockCapture();
    const asked: Array<Record<string, unknown>> = [];

    jest.spyOn(weather, "cutTo").mockResolvedValue();
    jest.spyOn(weather, "release").mockResolvedValue();
    jest.spyOn(render, "captureMap").mockResolvedValue({ outcome: "completed", width: 2048, height: 2048 });
    mockIpcCommands({
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
      [ELevelsCommand.WRITE_MAP_ANNOTATIONS]: ({ request }) => {
        asked.push(request);

        return mockSessionSnapshot("<svg/>", "session");
      },
    });

    service.open();
    await service.capture();
    await service.drawAnnotations();

    expect(asked[0]).toEqual({
      path: PATH,
      area: { minX: -600, minZ: -615, maxX: 600, maxZ: 585 },
      kinds: [],
      isLabelled: false,
    });
    expect(service.annotationsUrl).toBe("blob:map");

    service.setMapAnnotations({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: true });
    await service.drawAnnotations();

    expect(asked.at(-1)).toMatchObject({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: true });
  });

  it("writes the annotations in the order drawn and exports once the last is written, the capture held meanwhile", async () => {
    jest.spyOn(LevelViewerCaptureService.prototype, "sessionId", "get").mockReturnValue("session");

    const { service, render, weather } = mockCapture();
    const events: Array<string> = [];
    const firstWrite: { finish: () => void } = { finish: () => undefined };

    jest.spyOn(weather, "cutTo").mockResolvedValue();
    jest.spyOn(weather, "release").mockResolvedValue();
    jest.spyOn(render, "captureMap").mockResolvedValue({ outcome: "completed", width: 2048, height: 2048 });
    jest.mocked(save).mockResolvedValueOnce("C:/out/map_zaton.dds");
    mockIpcCommands({
      [ELevelsCommand.FIND_SCREENSHOT_PATH]: PATH,
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

    service.open();
    await service.capture();
    service.setMapAnnotations({ kinds: [ELevelAnnotationKind.GRAPH_POINTS], isLabelled: false });

    const exporting: Promise<void> = service.exportAs() as unknown as Promise<void>;

    await waitFor(() => expect(service.isExporting).toBe(true));
    service.back();

    expect(service.captured).not.toBeNull();

    firstWrite.finish();
    await exporting;

    expect(events).toEqual(["write 0", "written 0", "write 1", "export"]);
    expect(service.exported).toBe("C:/out/map_zaton.dds");
  });

  it("keeps the parameters and shows no map where the capture was stopped", async () => {
    const { service, render, weather } = mockCapture();

    jest.spyOn(weather, "cutTo").mockResolvedValue();
    jest.spyOn(weather, "release").mockResolvedValue();
    jest.spyOn(render, "captureMap").mockResolvedValue({ outcome: "cancelled", width: 2048, height: 2048 });

    service.open();
    await service.capture();

    expect(service.captured).toBeNull();
    expect(service.mapParameters).not.toBeNull();
  });

  it("captures a cubemap around the camera under the view's own weather, its clock held", async () => {
    const { service, render, weather } = mockCapture();
    const cutTo = jest.spyOn(weather, "cutTo").mockResolvedValue();
    const release = jest.spyOn(weather, "release").mockResolvedValue();
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
    expect(service.captured?.mode).toBe(ELevelCaptureMode.CUBEMAP);
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
