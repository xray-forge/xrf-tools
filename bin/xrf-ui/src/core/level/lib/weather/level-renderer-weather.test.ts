import { describe, expect, it } from "@jest/globals";

import {
  toLevelRendererRain,
  toLevelRendererWeather,
  toLevelRendererWeatherBase,
  toLevelRendererWeatherKeyframe,
  toLevelRendererWeatherModifier,
} from "@/core/level/lib/weather/level-renderer-weather";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import {
  mockLevelWeatherCycle,
  mockLevelWeatherDescription,
  mockWeatherDescriptor,
} from "@/fixtures/mocks/weather.mocks";

describe("toLevelRendererWeatherKeyframe", () => {
  it("hands the renderer the keyframe's sky, clouds, light, fog and wind in engine units", () => {
    const keyframe = toLevelRendererWeatherKeyframe(
      mockWeatherDescriptor({
        cloudsColor: [0.4, 0.4, 0.4, 0.8],
        cloudsRotation: 1.5,
        cloudsTexture: "sky\\clouds_1",
        sunDirection: null,
      })
    );

    expect(keyframe).toMatchObject({
      cloudsColor: [0.4, 0.4, 0.4, 0.8],
      cloudsRotation: 1.5,
      cloudsTexture: "sky\\clouds_1",
      skyTexture: "sky\\sky_noon",
      skyTextureEnv: "sky\\sky_noon#small",
      sunDirection: null,
      time: 43_200,
    });
  });

  // JSON has no NaN: a number the engine would hold as one crosses as null.
  it("reads a number the backend could not write as zero", () => {
    const keyframe = toLevelRendererWeatherKeyframe(
      mockWeatherDescriptor({ fogColor: [null, 0.5, 0.5], fogDistance: null })
    );

    expect(keyframe.fogColor).toEqual([0, 0.5, 0.5]);
    expect(keyframe.fogDistance).toBe(0);
  });
});

describe("toLevelRendererWeatherModifier", () => {
  it("takes a modifier as the level file holds it, every value flagged where the file is older than the flags", () => {
    expect(
      toLevelRendererWeatherModifier({
        ambient: { x: 0.1, y: 0.2, z: 0.3 },
        farPlane: 200,
        fogColor: { x: 1, y: null, z: 0 },
        fogDensity: 0.5,
        hemiColor: { x: 0, y: 0, z: 0 },
        position: { x: 10, y: 2, z: -30 },
        power: 1,
        radius: 25,
        skyColor: { x: 0, y: 0, z: 0 },
        useFlags: null,
      })
    ).toEqual({
      ambient: [0.1, 0.2, 0.3],
      farPlane: 200,
      flags: 0xffff,
      fogColor: [1, 0, 0],
      fogDensity: 0.5,
      hemiColor: [0, 0, 0],
      position: [10, 2, -30],
      power: 1,
      radius: 25,
      skyColor: [0, 0, 0],
    });
  });
});

describe("toLevelRendererRain", () => {
  it("hands the renderer the splash's mesh as the model holds it, by texture reference", () => {
    expect(
      toLevelRendererRain({
        drop: {
          indices: [0, 1, 2],
          positions: [0, null, 1, 1, 0, 0, 0, 0, 0],
          texture: { logicalPath: null, reference: "fx_splash" },
          uvs: [0, 0, 1, 0, 0, 1],
        },
        streak: { logicalPath: null, reference: "fx_rain" },
      })
    ).toEqual({
      drop: {
        indices: [0, 1, 2],
        positions: [0, 0, 1, 1, 0, 0, 0, 0, 0],
        texture: "fx_splash",
        uvs: [0, 0, 1, 0, 0, 1],
      },
      streak: "fx_rain",
    });
  });
});

describe("toLevelRendererWeather", () => {
  it("hands the renderer every effect and where their textures are fetched from, each texture once", async () => {
    const roots = mockSelectedLevelDescription().roots;
    const base = await toLevelRendererWeatherBase({
      description: mockLevelWeatherDescription({
        effects: [
          mockLevelWeatherCycle({
            keyframes: [mockWeatherDescriptor({ time: 0 })],
            name: "fx_blowout",
            textures: [
              { logicalPath: "textures\\sky\\sky_noon.dds", reference: "sky\\sky_noon" },
              { logicalPath: "textures\\sky\\blowout.dds", reference: "sky\\blowout" },
            ],
          }),
        ],
      }),
      roots,
    });
    const weather = await toLevelRendererWeather({ base, cycle: mockLevelWeatherCycle(), roots });

    expect(Object.keys(weather.effects)).toEqual(["fx_blowout"]);
    expect(Object.keys(weather.textures).sort()).toEqual(
      [
        "fx\\fx_rain",
        "sky\\blowout",
        "sky\\sky_night",
        "sky\\sky_night#small",
        "sky\\sky_noon",
        "water\\water_SBumpVolume",
        "water\\water_flowing_nmap",
      ].sort()
    );
    expect(weather.modifiers).toEqual([]);
    expect(weather.rain).toEqual({ drop: null, streak: "fx\\fx_rain" });
    expect(weather.wet).toEqual({ flow: "water\\water_flowing_nmap", splash: "water\\water_SBumpVolume" });
    // Built once for the level: every weather over it hands the renderer the same parts.
    expect(weather.thunder).toBe(base.thunder);
    expect(weather.effects).toBe(base.effects);
  });
});
