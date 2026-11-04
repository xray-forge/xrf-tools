import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import {
  ERenderAntialiasing,
  ERenderDebugView,
  ERenderLightLevel,
  ERenderOverlay,
  ERenderSurfaceColor,
} from "@xrf/ipc/types/xrf-renderer";

import { LEVEL_VIEWPORT, mockAttachedLevelRender, sentOverlayKinds } from "@/core/level/fixtures/level-render";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { LevelViewService } from "@/core/level/services/level-view";
import { ERenderResolution } from "@/core/render/lib/settings/device/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { SettingsService } from "@/core/settings/services/settings";
import { clearLocalStorage } from "@/core/storage";
describe("LevelRenderDrawing", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
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

  // The choice is kept, so it comes back with FSR, but the viewport is never left showing a target nothing lists.
  it("draws the final frame for an FSR view while FSR does not draw, and the view again once it does", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);
    const settings: RenderSettingsService = container.get(RenderSettingsService);

    settings.setRendererOverrides({ antialiasing: ERenderAntialiasing.FSR });
    view.setShading(ELevelShading.FSR_LOCKS);

    expect(service.drawing.shading).toBe(ELevelShading.FSR_LOCKS);

    settings.setRendererOverrides({ antialiasing: ERenderAntialiasing.TAA });

    expect(view.shading).toBe(ELevelShading.FSR_LOCKS);
    expect(service.drawing.shading).toBe(ELevelShading.FINAL);

    settings.setRendererOverrides({ antialiasing: ERenderAntialiasing.FSR });

    expect(service.drawing.shading).toBe(ELevelShading.FSR_LOCKS);
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

    expect(service.drawing.options.features.lightLevel).toBe(ERenderLightLevel.PHYSICAL);

    settings.setDevModeEnabled(false);

    expect(service.drawing.options.features.lightLevel).toBe(ERenderLightLevel.AUTHORED);
    expect(view.features.lightLevel).toBe(ERenderLightLevel.PHYSICAL);

    settings.setDevModeEnabled(true);

    expect(service.drawing.options.features.lightLevel).toBe(ERenderLightLevel.PHYSICAL);
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
});
