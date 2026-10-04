import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import {
  ERenderCamera,
  ERenderCameraCommand,
  ERenderDebugView,
  ERenderLevelHit,
  ERenderOverlay,
  ERenderSurfaceColor,
  ERenderViewportEvent,
  ERenderWeatherPlay,
  ERenderWeatherTransition,
  RenderCamera,
  RenderFrameReport,
  RenderLevelHit,
  RenderSurfaceSpan,
  RenderViewportEvent,
} from "@/core/ipc/types/xrf-renderer";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelRenderService, toLevelPick } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { SettingsService } from "@/core/settings/services/settings";
import { mockLevelSpawnObject, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockRenderLoadReport } from "@/fixtures/mocks/render.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import {
  getMockChannels,
  MockChannel,
  mockInvoke,
  resetMockChannels,
  resetMockInvoke,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription, mockRenderWeatherReport } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const VIEWPORT: number = 7;

const SPAN: RenderSurfaceSpan = { uMax: 2, uMin: 0, vMax: 1, vMin: -1 };

const REPORT: RenderFrameReport = {
  adapter: "Test GPU",
  backend: "D3D12",
  cpuTime: 0.8,
  frameTime: 6.25,
  frameTimeMax: 9,
  framesPerSecond: 160,
  height: 600,
  isGpuTimed: true,
  passes: [
    { gpuTime: 1.25, name: "g-buffer" },
    { gpuTime: 0.5, name: "sun" },
  ],
  memory: { scene: 4_194_304, textures: 67_108_864 },
  lights: {
    atlas: { capacity: 16_777_216, used: 4_194_304 },
    dropped: 3,
    excess: 0,
    fullClusters: 2,
    inView: 12,
    shadowed: 4,
  },
  particles: { drawn: 9, effects: 14, particles: 210, simulated: 11, simulationTime: 0.3 },
  renderHeight: 400,
  renderWidth: 534,
  sectorTime: 1.5,
  staticDraws: {
    clusters: { capacity: 4096, used: 3000 },
    commands: 13,
    keptClusters: 1200,
    keptTriangles: 90_000,
    lods: { capacity: 64, used: 40 },
    occludedClusters: 300,
    occludedTriangles: 20_000,
    places: { capacity: 2048, used: 1500 },
    rows: { capacity: 512, used: 200 },
    slots: { capacity: 1024, used: 900 },
    surfaceList: { capacity: 8192, used: 1200 },
  },
  width: 800,
};

/** What the render commands were sent, in order, as `[command, arguments]`. */
function sent(command: string): Array<Record<string, unknown>> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === `plugin:render|${command}`)
    .map(([, args]) => args as Record<string, unknown>);
}

/** The kinds of the overlays last sent, in order. */
function sentOverlayKinds(): Array<string> {
  return ((sent("set_overlays").at(-1)?.overlays ?? []) as Array<{ kind: string }>).map(({ kind }) => kind);
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

async function mockAttached(
  element: HTMLElement = document.createElement("div")
): Promise<{ container: Container; service: LevelRenderService }> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(
      mockSelectedLevelDescription({ start: { direction: { x: 0, y: 0, z: -1 }, position: { x: 10, y: 2, z: -5 } } })
    ),
    ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    ["plugin:render|attach_viewport"]: VIEWPORT,
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelWeatherService,
    LevelLookService,
    LevelRenderService,
  ]);

  await container.get(LevelLoadService).restore();

  const service: LevelRenderService = container.get(LevelRenderService);

  service.attach(element);
  await flush();

  return { container, service };
}

function emit(event: RenderViewportEvent): void {
  (getMockChannels()[0] as MockChannel<RenderViewportEvent>).onmessage(event);
}

describe("LevelRenderService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    window.localStorage.clear();
  });

  it("attaches a native viewport to its window and stands the camera at the level's start", async () => {
    await mockAttached();

    expect(sent("attach_viewport")).toEqual([{ events: getMockChannels()[0], window: "main" }]);
    expect(sent("configure")).toEqual([{ settings: { frameRate: { isVsync: true, limit: null }, isGpuTimed: false } }]);

    const camera: RenderCamera = sent("set_camera").at(-1)?.camera as RenderCamera;

    expect(sent("set_camera").at(-1)?.viewport).toBe(VIEWPORT);
    expect(camera.kind).toBe(ERenderCamera.FLY);
    expect(sent("command_camera").at(-1)).toEqual({
      command: { kind: ERenderCameraCommand.RESET },
      viewport: VIEWPORT,
    });
  });

  it("shows the open level in its viewport and reveals it once the renderer has it resident", async () => {
    const { container } = await mockAttached();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    expect(sent("show_level")).toEqual([{ sessionId: expect.any(String), viewport: VIEWPORT }]);
    expect(viewport.isRevealed).toBe(false);

    emit({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({
        bytes: 10,
        isReady: false,
        sectors: 1,
        sectorsTotal: 2,
        textures: 0,
        texturesTotal: 3,
      }),
    });
    expect(viewport.isRevealed).toBe(false);

    setMockInvokeResponses({
      ["plugin:render|describe_textures"]: [
        {
          reference: "terrain\\terrain_escape",
          state: { height: 512, isExpanded: false, kind: "loaded", layout: "DXT1", levels: 10, width: 1024 },
        },
        { reference: "act\\act_none", state: { kind: "missing" } },
      ],
      ["plugin:render|measure_surfaces"]: [{ drawables: 3, narrowest: SPAN, shaderId: 7, span: SPAN, triangles: 40 }],
    });
    emit({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({
        bytes: 20,
        isReady: true,
        sectors: 2,
        sectorsTotal: 2,
        textures: 3,
        texturesTotal: 3,
      }),
    });
    expect(viewport.isRevealed).toBe(true);

    // Counted once everything is resident, so it counts the level whole.
    await waitFor(() => expect(viewport.surfaceGeometry.get(7)?.triangles).toBe(40));
    expect(viewport.surfaceGeometry.get(7)).toEqual({ drawables: 3, narrowest: SPAN, span: SPAN, triangles: 40 });
    expect(viewport.textureReport.uploaded).toBe(2);
    expect(viewport.textureReport.dressing.get("terrain\\terrain_escape")?.upload).toBe("1024×512 · DXT1 · 10 levels");
    expect(viewport.textureReport.problems).toEqual([
      { reason: "Nothing in the mounted roots answers to it", reference: "act\\act_none" },
    ]);
  });

  it("reads frames and the camera into the readouts, and forgets the frame once detached", async () => {
    const { container, service } = await mockAttached();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    emit({ kind: ERenderViewportEvent.FRAME, report: REPORT });
    expect(service.frame).toBe(REPORT);

    emit({ kind: ERenderViewportEvent.CAMERA, pose: { position: [1, 2, 3], target: [1, 2, 2] } });
    // The readout states the engine's space, which mirrors renderer space along z.
    expect(viewport.camera?.position).toEqual({ x: 1, y: 2, z: -3 });

    service.detach();
    expect(service.frame.framesPerSecond).toBe(0);
  });

  it("goes to a place by standing the camera there anew", async () => {
    const { service } = await mockAttached();

    mockInvoke.mockClear();
    service.goTo({ heading: 90, pitch: 0, x: 4, y: 5, z: 6 });
    await flush();

    expect(sent("set_camera")).toHaveLength(1);
    expect(sent("command_camera")).toEqual([{ command: { kind: ERenderCameraCommand.RESET }, viewport: VIEWPORT }]);
  });

  it("keeps where the camera flew when the toolbar changes its speed", async () => {
    const { container } = await mockAttached();
    const view: LevelViewService = container.get(LevelViewService);

    mockInvoke.mockClear();
    view.setCamera({ ...view.camera, speed: view.camera.speed * 2 });
    await flush();

    expect(sent("set_camera")).toHaveLength(1);
    expect(sent("command_camera")).toHaveLength(0);
  });

  it("draws with what the toolbar and the settings come to", async () => {
    const { container } = await mockAttached();
    const view: LevelViewService = container.get(LevelViewService);

    expect(sent("set_view_options").at(-1)).toEqual({
      options: expect.objectContaining({
        isBumped: true,
        surfaceColor: ERenderSurfaceColor.TEXTURED,
        // The settings' level of detail, the engine's own thresholds by default.
        lod: expect.objectContaining({ isImpostors: true, ssaA: 64, ssaB: 48, ssaDiscard: 3.5 }),
      }),
      viewport: VIEWPORT,
    });

    mockInvoke.mockClear();
    view.setShading(ELevelShading.CLAY);
    await flush();

    expect(sent("set_view_options")).toEqual([
      {
        options: expect.objectContaining({ debugView: ERenderDebugView.FINAL, surfaceColor: ERenderSurfaceColor.CLAY }),
        viewport: VIEWPORT,
      },
    ]);
  });

  it("draws as a wireframe and at the height the settings ask for", async () => {
    const { container } = await mockAttached();
    const view: LevelViewService = container.get(LevelViewService);

    expect(sent("set_view_options").at(-1)).toEqual({
      options: expect.objectContaining({ isWireframe: false, renderHeight: null }),
      viewport: VIEWPORT,
    });

    view.setOptions({ ...view.options, isWireframe: true });
    container.get(SettingsService).setRenderResolution(ERenderResolution.HEIGHT_720);
    await flush();

    expect(sent("set_view_options").at(-1)).toEqual({
      options: expect.objectContaining({ isWireframe: true, renderHeight: 720 }),
      viewport: VIEWPORT,
    });
  });

  it("draws the grid, the extent, the axes and the sun over the level as the toolbar shows them", async () => {
    const { container } = await mockAttached();
    const view: LevelViewService = container.get(LevelViewService);

    view.setOptions({ ...view.options, isAxesVisible: false, isGridVisible: false, isSunMarked: true });
    await flush();

    expect(sentOverlayKinds()).toEqual([ERenderOverlay.SUN]);

    view.setOptions({ ...view.options, isAxesVisible: true, isGridVisible: true, isSunMarked: false });
    await flush();

    expect(sentOverlayKinds()).toEqual([
      ERenderOverlay.LINES,
      ERenderOverlay.LINES,
      ERenderOverlay.LINES,
      ERenderOverlay.LINES,
    ]);
  });

  it("plays the level's weather in its viewport, runs its clock as asked, and hears where it stands", async () => {
    const { container } = await mockAttached();
    const weather: LevelWeatherService = container.get(LevelWeatherService);

    await waitFor(() =>
      expect(sent("play_weather").at(-1)).toEqual({
        play: { kind: ERenderWeatherPlay.CYCLE, name: "default_clear" },
        transition: ERenderWeatherTransition.CUT,
        viewport: VIEWPORT,
      })
    );
    expect(sent("set_weather_control").at(-1)).toEqual({
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      viewport: VIEWPORT,
    });

    mockInvoke.mockClear();
    weather.setPlaying(true);
    weather.seekTo(3_600);
    weather.playEffect("fx_storm");
    weather.playAmbientEffect();
    weather.playAmbientEffect();
    await flush();

    expect(sent("set_weather_control")).toEqual([
      { control: { factor: 12, isDynamicSun: false, isPaused: false }, viewport: VIEWPORT },
    ]);
    expect(sent("seek_weather")).toEqual([{ time: 3_600, viewport: VIEWPORT }]);
    expect(sent("play_weather_effect")).toEqual([{ name: "fx_storm", viewport: VIEWPORT }]);
    expect(sent("play_ambient_effect")).toEqual([{ viewport: VIEWPORT }, { viewport: VIEWPORT }]);

    emit({ kind: ERenderViewportEvent.WEATHER, report: mockRenderWeatherReport({ time: 4_000 }) });
    expect(weather.time).toBe(4_000);
    expect(weather.report?.between).toEqual([0, 43_200]);
  });

  it("names what a pick hit in the level's own coordinates", async () => {
    const { container, service } = await mockAttached();
    const hit: RenderLevelHit = {
      isImpostor: false,
      kind: ERenderLevelHit.SURFACE,
      mesh: 2,
      place: 7,
      point: [1, 2, 3],
      sector: 4,
      shaderId: 11,
    };

    setMockInvokeResponses({ ["plugin:render|pick"]: hit });
    await service.pick({ x: 10, y: 20 });

    expect(sent("pick").at(-1)).toEqual({ viewport: VIEWPORT, x: 10, y: 20 });
    expect(container.get(LevelViewportService).picked).toEqual({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: 2,
      place: 7,
      point: { x: 1, y: 2, z: -3 },
      sector: 4,
      shaderId: 11,
    });
  });

  // The scene's keys act on it only while it has the keyboard, which leaving it or detaching takes away.
  it("notes whether the scene has the keyboard", async () => {
    const element: HTMLElement = document.createElement("div");
    const { container, service } = await mockAttached(element);
    const viewportService: LevelViewportService = container.get(LevelViewportService);

    element.dispatchEvent(new FocusEvent("focus"));
    expect(viewportService.isSceneFocused).toBe(true);

    element.dispatchEvent(new FocusEvent("blur"));
    expect(viewportService.isSceneFocused).toBe(false);

    element.dispatchEvent(new FocusEvent("focus"));
    service.detach();
    expect(viewportService.isSceneFocused).toBe(false);
  });

  it("names a picked spawned object as the held spawn has it, and nothing for one not held", () => {
    const crate = mockLevelSpawnObject({ index: 5, name: "crate_5", visual: 1 });
    const spawn = { objects: [mockLevelSpawnObject(), crate], visuals: ["lamp", "crate"] };
    const hit: RenderLevelHit = { kind: ERenderLevelHit.SPAWN, object: 5, point: [1, 2, 3] };

    expect(toLevelPick(hit, spawn)).toEqual({
      kind: ELevelPick.SPAWN,
      object: crate,
      point: { x: 1, y: 2, z: -3 },
      visual: "crate",
    });
    expect(toLevelPick({ ...hit, object: 9 }, spawn)).toBeNull();
    expect(toLevelPick(hit, null)).toBeNull();
  });

  it("states why the viewport cannot draw, and lets it go on detach", async () => {
    const { service } = await mockAttached();

    emit({ kind: ERenderViewportEvent.FAILURE, message: "No GPU" });
    expect(service.failure).toBe("No GPU");

    service.detach();
    await flush();

    expect(sent("detach_viewport")).toEqual([{ viewport: VIEWPORT }]);
  });
});
