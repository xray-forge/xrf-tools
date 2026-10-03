import { beforeEach, describe, expect, it } from "@jest/globals";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ERenderAntialiasing, ERenderFrameRate, ERenderScale } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_RENDER_FEATURE_CHOICE } from "@/core/render/lib/settings/render-feature-choice";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { SettingsService } from "@/core/settings/services/settings/settings.service";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("SettingsService", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("takes its first value from the build, which is on under test", () => {
    const { service } = mockInjectedService(SettingsService);

    // Jest runs with NODE_ENV=test, so `isDevelopmentBuild()` is true here for the same reason it is
    // true in a dev bundle: only a production build turns it off.
    expect(service.isDevModeEnabled).toBe(true);
  });

  it("lets a stored choice override the build default", () => {
    window.localStorage.setItem("xrf.preference.dev-mode", "false");

    const { service } = mockInjectedService(SettingsService);

    expect(service.isDevModeEnabled).toBe(false);
  });

  it("persists what was chosen, which is what makes tracing a release build possible", () => {
    const { service } = mockInjectedService(SettingsService);

    service.setDevModeEnabled(false);

    expect(service.isDevModeEnabled).toBe(false);
    expect(window.localStorage.getItem("xrf.preference.dev-mode")).toBe("false");
    expect(service.isDevModeEnabled).toBe(false);
  });

  it("falls back to the dense rows when nothing has chosen a catalog view", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.catalogView).toBe("rows");
  });

  it("refuses a catalog view this build does not know, rather than handing it to the launcher", () => {
    // Written by an older build, a hand edit, or a half-finished rename. `JSON.parse` would have
    // thrown on it here, taking the whole service down while it was being constructed.
    window.localStorage.setItem("xrf.preference.catalog-view", "spreadsheet");

    const { service } = mockInjectedService(SettingsService);

    expect(service.catalogView).toBe("rows");
  });

  it("gives back the catalog view it was told to keep", () => {
    const { service } = mockInjectedService(SettingsService);

    service.setCatalogView("grid");

    expect(service.catalogView).toBe("grid");
    expect(window.localStorage.getItem("xrf.preference.catalog-view")).toBe("grid");
    expect(mockInjectedService(SettingsService).service.catalogView).toBe("grid");
  });

  it("caps viewports at sixty until something says otherwise", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.frameRateLimit).toBe("60");
  });

  it("gives back the frame rate limit it was told to keep", () => {
    const { service } = mockInjectedService(SettingsService);

    service.setFrameRateLimit("30");

    expect(service.frameRateLimit).toBe("30");
    expect(window.localStorage.getItem("xrf.preference.frame-rate-limit")).toBe("30");
    expect(mockInjectedService(SettingsService).service.frameRateLimit).toBe("30");
  });

  it("reads configs as the vanilla engine until told otherwise, and keeps what it was told", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.engine).toBe(EXrayEngine.VANILLA);

    service.setEngine(EXrayEngine.EXTENDED);

    expect(window.localStorage.getItem("xrf.preference.engine")).toBe("extended");
    expect(mockInjectedService(SettingsService).service.engine).toBe(EXrayEngine.EXTENDED);
  });

  it("draws every viewport at most sixty frames a second until told otherwise, and keeps what it was told", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.renderSettings.frameRate).toEqual({ framesPerSecond: 60, kind: ERenderFrameRate.LIMITED });

    service.setFrameRateLimit("unlimited");

    expect(window.localStorage.getItem("xrf.preference.frame-rate-limit")).toBe("unlimited");
    expect(mockInjectedService(SettingsService).service.renderSettings.frameRate).toEqual({
      kind: ERenderFrameRate.UNLIMITED,
    });
  });

  // Timing is asked for, never a preset's: turning it on leaves the settings as the preset draws them.
  it("times no passes until told to, and keeps it apart from the preset", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.renderSettings.isGpuTimed).toBe(false);

    service.setGpuTimed(true);

    expect(service.rendererChoice).toEqual(DEFAULT_RENDER_FEATURE_CHOICE);
    expect(window.localStorage.getItem("xrf.preference.gpu-timed")).toBe("true");
    expect(mockInjectedService(SettingsService).service.renderSettings.isGpuTimed).toBe(true);
  });

  it("draws with Base until another preset is chosen, and keeps what was changed on top of it", () => {
    const { service } = mockInjectedService(SettingsService);

    expect(service.rendererFeatures).toEqual(RENDER_PRESETS[ERenderPreset.BASE]);

    service.setRendererOverrides({ antialiasing: ERenderAntialiasing.FXAA });
    service.setRendererOverrides({ lod: { ssaA: 80 } });
    service.setRendererOverrides({ lod: { ssaB: 40 } });

    const reloaded = mockInjectedService(SettingsService).service;

    expect(reloaded.rendererFeatures.antialiasing).toBe(ERenderAntialiasing.FXAA);
    expect([reloaded.rendererFeatures.lod.ssaA, reloaded.rendererFeatures.lod.ssaB]).toEqual([80, 40]);

    // A preset chosen again is the preset whole: whatever was changed on the last one goes.
    reloaded.setRendererPreset(ERenderPreset.EDITING);

    expect(reloaded.rendererFeatures).toEqual(RENDER_PRESETS[ERenderPreset.EDITING]);
    expect(mockInjectedService(SettingsService).service.rendererChoice.preset).toBe(ERenderPreset.EDITING);
  });

  // Two writes to one group keep both, whichever group: a group forgotten used to be replaced whole.
  it("keeps every group's earlier changes when another of its settings changes", () => {
    const { service } = mockInjectedService(SettingsService);

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
    window.localStorage.setItem("xrf.preference.renderer-features", "{not json");

    expect(mockInjectedService(SettingsService).service.rendererChoice.preset).toBe(ERenderPreset.BASE);
  });
});
