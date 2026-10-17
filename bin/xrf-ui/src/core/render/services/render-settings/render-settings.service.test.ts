import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";

import {
  ERenderAntialiasing,
  ERenderScale,
  ERenderTextureFiltering,
  RENDER_DEFAULT_PERIODIC_BUDGET,
} from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_RENDER_FEATURE_CHOICE } from "@/core/render/lib/settings/render-feature-choice";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { RenderSettingsService } from "@/core/render/services/render-settings/render-settings.service";
import { clearLocalStorage, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { mockInvoke } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer, mockInjectedService } from "@/fixtures/utils/container";

/** How often the renderer was told its settings. */
function countConfigured(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === "plugin:render|configure").length;
}

describe("RenderSettingsService", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  // The settings are the renderer's, not a viewport's: told once, then once a change, however many viewports draw.
  it("tells the renderer its settings while provisioned, with no viewport attached", () => {
    const container: Container = mockContainer();
    const service: RenderSettingsService = container.get(RenderSettingsService);

    mockInvoke.mockClear();
    container.provision();

    expect(countConfigured()).toBe(1);

    service.setVsync(false);

    expect(countConfigured()).toBe(2);

    container.deprovision();
    service.setVsync(true);

    expect(countConfigured()).toBe(2);
  });

  it("tells the renderer each change once when provisioned again", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    service.onProvision();
    service.onProvision();
    mockInvoke.mockClear();
    service.setVsync(false);

    expect(countConfigured()).toBe(1);

    service.onDeprovision();
  });

  // Every viewport samples the renderer's one texture cache, so its filtering and quality are the renderer's settings.
  it("tells the renderer the textures' filtering and quality as its own settings, not a viewport's", () => {
    const container: Container = mockContainer();
    const service: RenderSettingsService = container.get(RenderSettingsService);

    container.provision();
    mockInvoke.mockClear();
    service.setRendererOverrides({ textureFiltering: ERenderTextureFiltering.TRILINEAR, textureLod: 2 });

    expect(countConfigured()).toBe(1);
    expect(service.renderSettings).toMatchObject({
      textureFiltering: ERenderTextureFiltering.TRILINEAR,
      textureLod: 2,
    });

    container.deprovision();
  });

  it("allows viewports any frame rate until something says otherwise", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    expect(service.frameRateLimit).toBe("none");
  });

  it("gives back the frame rate limit it was told to keep", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    service.setFrameRateLimit("30");

    expect(service.frameRateLimit).toBe("30");
    expect(getLocalStorageValue("xrf.preference.frame-rate-limit")).toBe("30");
    expect(mockInjectedService(RenderSettingsService).service.frameRateLimit).toBe("30");
  });

  // No cap under vsync is the display's own refresh rate, whatever it is, with nothing to detect.
  it("draws every viewport at the display's refresh until told otherwise, and keeps what it was told", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    expect(service.renderSettings.frameRate).toEqual({ isVsync: true, limit: null });

    service.setFrameRateLimit("120");
    service.setVsync(false);

    expect(getLocalStorageValue("xrf.preference.vsync")).toBe("false");
    expect(mockInjectedService(RenderSettingsService).service.renderSettings.frameRate).toEqual({
      isVsync: false,
      limit: 120,
    });
  });

  it("reads a frame rate limit it does not offer as no cap", () => {
    setLocalStorageValue("xrf.preference.frame-rate-limit", "unlimited");

    expect(mockInjectedService(RenderSettingsService).service.frameRateLimit).toBe("none");
  });

  // Timing is asked for, never a preset's: turning it on leaves the settings as the preset draws them.
  it("times no passes until told to, and keeps it apart from the preset", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    expect(service.renderSettings.isGpuTimed).toBe(false);

    service.setGpuTimed(true);

    expect(service.rendererChoice).toEqual(DEFAULT_RENDER_FEATURE_CHOICE);
    expect(getLocalStorageValue("xrf.preference.gpu-timed")).toBe("true");
    expect(mockInjectedService(RenderSettingsService).service.renderSettings.isGpuTimed).toBe(true);
  });

  // The budget is the renderer's own unless set for a session's measurement.
  it("gives periodic work the renderer's budget until set, for the session only", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    expect(service.renderSettings.periodicBudget).toBe(RENDER_DEFAULT_PERIODIC_BUDGET);

    service.setPeriodicBudget(0.8);

    expect(service.renderSettings.periodicBudget).toBe(0.8);
    expect(mockInjectedService(RenderSettingsService).service.renderSettings.periodicBudget).toBe(
      RENDER_DEFAULT_PERIODIC_BUDGET
    );
  });

  it("draws with Base until another preset is chosen, and keeps what was changed on top of it", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    expect(service.rendererFeatures).toEqual(RENDER_PRESETS[ERenderPreset.BASE]);

    service.setRendererOverrides({ antialiasing: ERenderAntialiasing.FXAA });
    service.setRendererOverrides({ lod: { ssaA: 80 } });
    service.setRendererOverrides({ lod: { ssaB: 40 } });

    const reloaded = mockInjectedService(RenderSettingsService).service;

    expect(reloaded.rendererFeatures.antialiasing).toBe(ERenderAntialiasing.FXAA);
    expect([reloaded.rendererFeatures.lod.ssaA, reloaded.rendererFeatures.lod.ssaB]).toEqual([80, 40]);

    // A preset chosen again is the preset whole: whatever was changed on the last one goes.
    reloaded.setRendererPreset(ERenderPreset.EDITING);

    expect(reloaded.rendererFeatures).toEqual(RENDER_PRESETS[ERenderPreset.EDITING]);
    expect(mockInjectedService(RenderSettingsService).service.rendererChoice.preset).toBe(ERenderPreset.EDITING);
  });

  // Two writes to one group keep both, whichever group.
  it("keeps every group's earlier changes when another of its settings changes", () => {
    const { service } = mockInjectedService(RenderSettingsService);

    service.setRendererOverrides({ grass: { radius: 80 }, shadows: { bias: 2 }, upscaling: { sharpening: 0.2 } });
    service.setRendererOverrides({
      grass: { height: 1.5 },
      shadows: { filter: 2 },
      upscaling: { scale: ERenderScale.QUALITY },
    });

    expect(service.rendererChoice.overrides).toEqual({
      grass: { height: 1.5, radius: 80 },
      shadows: { bias: 2, filter: 2 },
      upscaling: { scale: ERenderScale.QUALITY, sharpening: 0.2 },
    });
  });

  it("falls back to Base for a stored choice that does not parse", () => {
    setLocalStorageValue("xrf.preference.renderer-features", "{not json");

    expect(mockInjectedService(RenderSettingsService).service.rendererChoice.preset).toBe(ERenderPreset.BASE);
  });
});
