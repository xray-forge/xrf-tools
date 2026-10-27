import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { ERenderDebugView, ERenderLightLevel, ERenderOverlay, ERenderSurfaceColor } from "@xrf/ipc/types/xrf-renderer";
import {
  EWorldCameraCommand,
  EWorldViewportEvent,
  EWorldWeatherPlay,
  EWorldWeatherTransition,
} from "@xrf/ipc/types/xrf-world";

import {
  emitViewportEvent,
  LEVEL_VIEWPORT,
  mockAttachedLevelRender,
  sentOverlayKinds,
} from "@/core/level/fixtures/level-render";
import { mockWorldWeatherReport } from "@/core/level/fixtures/weather.mocks";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { LevelViewService } from "@/core/level/services/level-view/level-view.service";
import { LevelWeatherService } from "@/core/level/services/level-weather/level-weather.service";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { SettingsService } from "@/core/settings/services/settings";
import { clearLocalStorage } from "@/core/storage";

describe("LevelRenderService view", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("goes to a place by standing the camera there anew", async () => {
    const { service } = await mockAttachedLevelRender();

    mockInvoke.mockClear();
    service.goTo({ heading: 90, pitch: 0, x: 4, y: 5, z: 6 });

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA)).toHaveLength(1));
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toEqual([
      { command: { kind: EWorldCameraCommand.RESET }, viewport: LEVEL_VIEWPORT },
    ]);
  });

  it("keeps where the camera flew when the toolbar changes its speed", async () => {
    const { container } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    mockInvoke.mockClear();
    view.setCameraOptions({ ...view.cameraOptions, speed: view.cameraOptions.speed * 2 });

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA)).toHaveLength(1));
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toHaveLength(0);
  });

  it("draws with what the toolbar and the settings come to", async () => {
    const { container } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)).toEqual({
      options: expect.objectContaining({
        // The settings' level of detail: the impostors twice as far as the engine's by default, the rest its own.
        features: expect.objectContaining({
          lod: expect.objectContaining({ isImpostors: true, ssaA: 32, ssaB: 24, ssaDiscard: 3.5 }),
        }),
        mode: expect.objectContaining({ isBumped: true, surfaceColor: ERenderSurfaceColor.TEXTURED }),
      }),
      viewport: LEVEL_VIEWPORT,
    });

    mockInvoke.mockClear();
    view.setShading(ELevelShading.CLAY);

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS)).not.toHaveLength(0));
    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS)).toEqual([
      {
        options: expect.objectContaining({
          mode: expect.objectContaining({ debugView: ERenderDebugView.FINAL, surfaceColor: ERenderSurfaceColor.CLAY }),
        }),
        viewport: LEVEL_VIEWPORT,
      },
    ]);
  });

  it("lets the world play what the toolbar switches, apart from what is drawn", async () => {
    const { container } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)).toEqual({
      toggles: expect.objectContaining({ isRainy: true, isSpawnedLamps: true }),
      viewport: LEVEL_VIEWPORT,
    });
    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)).not.toHaveProperty("options.world");

    mockInvoke.mockClear();
    view.setOptions({ ...view.options, isRainy: false, isSpawnedLamps: false });

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES)).not.toHaveLength(0));
    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES)).toEqual([
      { toggles: expect.objectContaining({ isRainy: false, isSpawnedLamps: false }), viewport: LEVEL_VIEWPORT },
    ]);
    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS)).toEqual([]);
  });

  it("draws as a wireframe and at the height the settings ask for", async () => {
    const { container } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)).toEqual({
      options: expect.objectContaining({
        mode: expect.objectContaining({ isWireframe: false }),
        output: expect.objectContaining({ outputHeight: null, outputScale: 1 }),
      }),
      viewport: LEVEL_VIEWPORT,
    });

    view.setOptions({ ...view.options, isWireframe: true });
    container.get(RenderSettingsService).setRenderResolution(ERenderResolution.HEIGHT_720);

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)).toEqual({
        options: expect.objectContaining({
          mode: expect.objectContaining({ isWireframe: true }),
          output: expect.objectContaining({ outputHeight: 720 }),
        }),
        viewport: LEVEL_VIEWPORT,
      })
    );
  });

  it("resolves at the share of the viewport the settings' scale asks while the resolution is automatic", async () => {
    const { container } = await mockAttachedLevelRender();

    container.get(RenderSettingsService).setRenderOutputScale(1.5);

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)).toEqual({
        options: expect.objectContaining({
          output: expect.objectContaining({ outputHeight: null, outputScale: 1.5 }),
        }),
        viewport: LEVEL_VIEWPORT,
      })
    );
  });

  // A view's own physical light level is parked as the settings' is: drawn at the authored levels outside dev mode,
  // and kept for when dev mode returns.
  it("draws a view's own physical light level as the authored one outside dev mode", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const settings: SettingsService = container.get(SettingsService);
    const view: LevelViewService = container.get(LevelViewService);

    settings.setDevModeEnabled(true);
    view.setFeatures({ ...view.features, lightLevel: ERenderLightLevel.PHYSICAL });

    expect(service.viewOptions.features.lightLevel).toBe(ERenderLightLevel.PHYSICAL);

    settings.setDevModeEnabled(false);

    expect(service.viewOptions.features.lightLevel).toBe(ERenderLightLevel.AUTHORED);
    expect(view.features.lightLevel).toBe(ERenderLightLevel.PHYSICAL);

    settings.setDevModeEnabled(true);

    expect(service.viewOptions.features.lightLevel).toBe(ERenderLightLevel.PHYSICAL);
  });

  it("draws the grid, the extent, the axes and the sun over the level as the toolbar shows them", async () => {
    const { container } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    view.setOptions({ ...view.options, isAxesVisible: false, isGridVisible: false, isSunMarked: true });

    await waitFor(() => expect(sentOverlayKinds()).toEqual([ERenderOverlay.SUN]));

    view.setOptions({ ...view.options, isAxesVisible: true, isGridVisible: true, isSunMarked: false });

    await waitFor(() => expect(sentOverlayKinds()).not.toContain(ERenderOverlay.SUN));
    expect(sentOverlayKinds()).toEqual([
      ERenderOverlay.LINES,
      ERenderOverlay.LINES,
      ERenderOverlay.LINES,
      ERenderOverlay.LINES,
    ]);
  });

  it("plays the level's weather in its viewport, runs its clock as asked, and hears where it stands", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const weather: LevelWeatherService = container.get(LevelWeatherService);

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.PLAY_WEATHER).at(-1)).toEqual({
        play: { kind: EWorldWeatherPlay.CYCLE, name: "default_clear" },
        transition: EWorldWeatherTransition.CUT,
        viewport: LEVEL_VIEWPORT,
      })
    );
    expect(listMockInvokeCalls(ERenderCommand.SET_WEATHER_CONTROL).at(-1)).toEqual({
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      viewport: LEVEL_VIEWPORT,
    });

    mockInvoke.mockClear();
    weather.setPlaying(true);
    service.seekWeather(3_600);
    // The same time again seeks again: the clock has run on since.
    service.seekWeather(3_600);
    service.playWeatherEffect("fx_storm");
    service.playAmbientEffect();
    service.playAmbientEffect();

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.PLAY_AMBIENT_EFFECT)).toHaveLength(2));
    expect(listMockInvokeCalls(ERenderCommand.SET_WEATHER_CONTROL)).toEqual([
      { control: { factor: 12, isDynamicSun: false, isPaused: false }, viewport: LEVEL_VIEWPORT },
    ]);
    expect(listMockInvokeCalls(ERenderCommand.SEEK_WEATHER)).toEqual([
      { time: 3_600, viewport: LEVEL_VIEWPORT },
      { time: 3_600, viewport: LEVEL_VIEWPORT },
    ]);
    expect(weather.time).toBe(3_600);
    expect(listMockInvokeCalls(ERenderCommand.PLAY_WEATHER_EFFECT)).toEqual([
      { name: "fx_storm", viewport: LEVEL_VIEWPORT },
    ]);
    expect(listMockInvokeCalls(ERenderCommand.PLAY_AMBIENT_EFFECT)).toEqual([
      { viewport: LEVEL_VIEWPORT },
      { viewport: LEVEL_VIEWPORT },
    ]);

    emitViewportEvent({ kind: EWorldViewportEvent.WEATHER, report: mockWorldWeatherReport({ time: 4_000 }) });
    expect(weather.time).toBe(4_000);
    expect(weather.report?.between).toEqual([0, 43_200]);
  });
});
