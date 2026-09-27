import { beforeEach, describe, expect, it } from "@jest/globals";
import { ERendererAntialiasing } from "@xrf/renderer";

import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { DEFAULT_LEVEL_FEATURE_OPTIONS } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("LevelViewService", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("starts with the defaults", () => {
    const { service } = mockInjectedService(LevelViewService);

    expect(service.options).toEqual(DEFAULT_LEVEL_VIEW_OPTIONS);
    expect(service.lighting).toEqual(DEFAULT_LEVEL_LIGHTING);
    expect(service.camera).toEqual(DEFAULT_LEVEL_CAMERA_OPTIONS);
  });

  it("takes what the viewer switched on", () => {
    const { service } = mockInjectedService(LevelViewService);

    service.setOptions({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isGridVisible: !DEFAULT_LEVEL_VIEW_OPTIONS.isGridVisible });
    service.setLighting({ ...DEFAULT_LEVEL_LIGHTING, sunIntensity: 4 });
    service.setCamera({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, fieldOfView: 40 });

    expect(service.options.isGridVisible).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isGridVisible);
    expect(service.lighting.sunIntensity).toBe(4);
    expect(service.camera.fieldOfView).toBe(40);
  });

  // The renderer may be on a thread of its own, and a value posted to it is structured-cloned. A deep observable
  // is a proxy, which cannot be cloned at all: holding these by reference is what lets the view cross the thread.
  it("holds what it was given rather than a copy that could not cross a thread", () => {
    const { service } = mockInjectedService(LevelViewService);

    const options: ILevelViewOptions = { ...DEFAULT_LEVEL_VIEW_OPTIONS };
    const lighting: ILevelLighting = { ...DEFAULT_LEVEL_LIGHTING };
    const camera: ILevelCameraOptions = { ...DEFAULT_LEVEL_CAMERA_OPTIONS };

    service.setOptions(options);
    service.setLighting(lighting);
    service.setCamera(camera);

    expect(service.options).toBe(options);
    expect(service.lighting).toBe(lighting);
    expect(service.camera).toBe(camera);

    expect(() =>
      structuredClone({ camera: service.camera, lighting: service.lighting, options: service.options })
    ).not.toThrow();
  });

  it("forgets the last level's toggles and keeps the camera and the features", () => {
    const { service } = mockInjectedService(LevelViewService);
    const camera: ILevelCameraOptions = { ...DEFAULT_LEVEL_CAMERA_OPTIONS, fieldOfView: 40 };

    service.setOptions({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isGridVisible: !DEFAULT_LEVEL_VIEW_OPTIONS.isGridVisible });
    service.setCamera(camera);
    service.setFeatures({ ...DEFAULT_LEVEL_FEATURE_OPTIONS, shadows: { filter: 0 } });
    service.clear();

    expect(service.options).toEqual(DEFAULT_LEVEL_VIEW_OPTIONS);
    expect(service.camera).toEqual(camera);
    expect(service.features.shadows).toEqual({ filter: 0 });
  });

  it("starts from the camera and the features the last run stored", () => {
    const first = mockInjectedService(LevelViewService).service;

    first.setCamera({ ...DEFAULT_LEVEL_CAMERA_OPTIONS, speed: 30 });
    first.setFeatures({ ...DEFAULT_LEVEL_FEATURE_OPTIONS, antialiasing: ERendererAntialiasing.TAA });

    const { service } = mockInjectedService(LevelViewService);

    expect(service.camera.speed).toBe(30);
    expect(service.features.antialiasing).toBe(ERendererAntialiasing.TAA);
  });

  it("starts from the defaults where what was stored does not read", () => {
    window.localStorage.setItem("xrf.preference.level-camera", "{not json");
    window.localStorage.setItem("xrf.preference.level-features", JSON.stringify({ antialiasing: "none", grass: 4 }));

    const { service } = mockInjectedService(LevelViewService);

    expect(service.camera).toEqual(DEFAULT_LEVEL_CAMERA_OPTIONS);
    expect(service.features).toEqual(DEFAULT_LEVEL_FEATURE_OPTIONS);
  });
});
