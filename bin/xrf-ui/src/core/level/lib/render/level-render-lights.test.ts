import { describe, expect, it } from "@jest/globals";
import { ERendererLightKind, IRendererLights } from "@xrf/renderer";

import { LevelLightsDescription } from "@/core/ipc/types/xrf-app";
import { ELightKind, LightDescription } from "@/core/ipc/types/xrf-visual";
import { toLevelRendererLights } from "@/core/level/lib/render/level-render-lights";

const SPOT: LightDescription = {
  animator: 0,
  animatorScale: 2 / 255,
  color: [0.5, 0.25, 1],
  cone: 2,
  direction: { x: 0, y: -1, z: 0 },
  isLevel: false,
  isShadowed: true,
  kind: ELightKind.SPOT,
  name: "lamp",
  near: 0.1,
  position: { x: 1, y: 2, z: 3 },
  projector: 1,
  range: 8,
  rangeJitter: 0,
  right: { x: 1, y: 0, z: 0 },
};

describe("toLevelRendererLights", () => {
  it("names a spot's projector by its texture reference, and keeps each animator's keys as frames and colours", () => {
    const description: LevelLightsDescription = {
      lights: {
        animators: [
          {
            fps: 15,
            frameCount: 30,
            keys: [
              { color: [255, 0, 0], frame: 0 },
              { color: [0, 128, null], frame: 20 },
            ],
            name: "light\\idle",
          },
        ],
        lights: [SPOT, { ...SPOT, animator: null, kind: ELightKind.POINT, projector: null }],
        projectors: ["lights\\lights_spot01", "lights\\lights_spot_wire_02"],
      },
      projectors: [],
    };
    const lights: IRendererLights = toLevelRendererLights(description);

    expect(lights.animators).toEqual([
      {
        colors: [
          [255, 0, 0],
          [0, 128, 0],
        ],
        fps: 15,
        frameCount: 30,
        frames: [0, 20],
      },
    ]);
    expect(lights.lights[0]).toEqual({
      animator: 0,
      animatorScale: 2 / 255,
      color: [0.5, 0.25, 1],
      cone: 2,
      direction: [0, -1, 0],
      isLevel: false,
      isShadowed: true,
      kind: ERendererLightKind.SPOT,
      near: 0.1,
      position: [1, 2, 3],
      projector: "lights\\lights_spot_wire_02",
      range: 8,
      rangeJitter: 0,
      right: [1, 0, 0],
    });
    expect(lights.lights[1].kind).toBe(ERendererLightKind.POINT);
    expect(lights.lights[1].projector).toBeUndefined();
    expect(lights.lights[1].animator).toBeUndefined();
  });
});
