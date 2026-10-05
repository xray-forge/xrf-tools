import { LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EWeatherCycleKind, WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import { RenderWeatherReport } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_LEVEL_MANUAL_WEATHER, toLevelManualDescriptor } from "@/core/level/lib/weather/level-manual-weather";

/**
 * Creates a keyframe fixture, a clear noon unless told otherwise.
 *
 * @param overrides - Field values to override.
 * @returns A keyframe as the engine loads it.
 */
export function mockWeatherDescriptor(overrides: Partial<WeatherDescriptor> = {}): WeatherDescriptor {
  return {
    ambient: null,
    ambientColor: [0.02, 0.02, 0.02],
    bloomExposure: 0,
    bloomSkyIntensity: 0,
    bloomThreshold: 0,
    cloudsColor: [0, 0, 0, 0],
    cloudsRotation: 0,
    cloudsTexture: "",
    farPlane: 1000,
    fogColor: [0.5, 0.5, 0.5],
    fogDensity: 0.5,
    fogDistance: 900,
    hemiColor: [0.47, 0.37, 0.33, 1],
    hemiContrast: 0,
    hemiVibrance: 0,
    isSunFixed: false,
    rainColor: [0, 0, 0],
    rainDensity: 0,
    skyColor: [1, 1, 1],
    skyRotation: 0,
    skyTexture: "sky\\sky_noon",
    skyTextureEnv: "sky\\sky_noon#small",
    sun: null,
    sunAzimuth: 0,
    sunColor: [0.9, 0.84, 0.7],
    sunDirection: [0, -0.5, 0.866],
    sunShaftsIntensity: 0,
    thunderboltCollection: null,
    thunderboltDuration: 0,
    thunderboltPeriod: 0,
    time: 43_200,
    treeAmplitude: 0.005,
    treeRotation: 10,
    treeSpeed: 1,
    treeWave: [0.1, 0.01, 0.11],
    volumetricDistanceFactor: 0,
    volumetricIntensityFactor: 0,
    waterIntensity: 1,
    wetSurfaceFactor: 0,
    windDirection: 0,
    windVelocity: 0,
    ...overrides,
  };
}

/**
 * Creates a cycle fixture: a midnight and a noon.
 *
 * @param overrides - Field values to override.
 * @returns A cycle as the engine loads it.
 */
export function mockLevelWeatherCycle(overrides: Partial<LevelWeatherCycle> = {}): LevelWeatherCycle {
  return {
    file: "environment\\weathers\\default_clear.ltx",
    findings: [],
    keyframes: [
      mockWeatherDescriptor({ skyTexture: "sky\\sky_night", skyTextureEnv: "sky\\sky_night#small", time: 0 }),
      mockWeatherDescriptor(),
    ],
    kind: EWeatherCycleKind.CYCLE,
    name: "default_clear",
    ...overrides,
  };
}

/**
 * Creates a level weather fixture offering one cycle.
 *
 * @param overrides - Field values to override.
 * @returns What a level's weather reads as.
 */
export function mockLevelWeatherDescription(overrides: Partial<LevelWeatherDescription> = {}): LevelWeatherDescription {
  return {
    cycles: [],
    effects: [],
    engine: EXrayEngine.VANILLA,
    modifiers: [],
    offered: [mockLevelWeatherCycle()],
    suns: ["gradient1", "moon_halo_full"],
    skies: [
      { texture: { logicalPath: "textures\\sky\\sky_night.dds", reference: "sky\\sky_night" }, uses: 1 },
      { texture: { logicalPath: "textures\\sky\\sky_noon.dds", reference: "sky\\sky_noon" }, uses: 1 },
    ],
    clouds: [{ texture: { logicalPath: "textures\\sky\\sky_oblaka.dds", reference: "sky\\sky_oblaka" }, uses: 2 }],
    sunTable: null,
    thunderboltCollections: [],
    weather: { key: "default", level: "zaton", options: [{ cycle: "default_clear", graph: null, state: null }] },
    ...overrides,
  };
}

/**
 * Creates a renderer weather report fixture: noon of `default_clear`, halfway between two keyframes.
 *
 * @param overrides - Field values to override.
 * @returns Where the renderer's weather stands.
 */
export function mockRenderWeatherReport(overrides: Partial<RenderWeatherReport> = {}): RenderWeatherReport {
  const time: number = overrides.time ?? 43_200;

  return {
    ambient: null,
    between: [0, 43_200],
    current: toLevelManualDescriptor(DEFAULT_LEVEL_MANUAL_WEATHER, time),
    effect: null,
    modifiers: 0,
    time,
    weight: 0.5,
    ...overrides,
  };
}
