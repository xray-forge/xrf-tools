import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera } from "three/webgpu";

import { DEFAULT_RENDERER_LIGHTS_SETTINGS } from "#/contract/renderer-features";
import { ERendererLightKind, IRendererLight } from "#/contract/scene/renderer-lights";
import { adoptRendererConventions } from "#/internals/camera-conventions";
import { toSunSpecular } from "#/lighting/base-lighting";
import { LIGHT_VECTORS } from "#/scene/lights/light-record";
import { SceneLights } from "#/scene/lights/scene-lights";
import { RendererTextures } from "#/texture/renderer-textures";
import { LodUniforms } from "#/uniforms/lod-uniforms";

const POINT: IRendererLight = {
  animatorScale: 0,
  color: [0.5, 0.25, 1],
  cone: 0,
  direction: [0, 0, 1],
  isLevel: false,
  isShadowed: false,
  kind: ERendererLightKind.POINT,
  near: 0,
  position: [1, 2, -10],
  range: 4,
  right: [1, 0, 0],
};

/** A camera at the origin looking down `-z`, as a renderer's camera is before it moves. */
function createCamera(): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 0.1, 1000);

  adoptRendererConventions(camera);
  camera.updateMatrixWorld();

  return camera;
}

function createLights(): SceneLights {
  return new SceneLights(
    new RendererTextures(
      () => {},
      () => {}
    )
  );
}

function readRecord(lights: SceneLights, index: number): Array<number> {
  return Array.from(
    (lights.records.array as Float32Array).subarray(index * LIGHT_VECTORS * 4, (index + 1) * LIGHT_VECTORS * 4)
  );
}

describe("SceneLights", () => {
  it("writes a point in view space with the engine's falloff, colour and specular weight", () => {
    const lights: SceneLights = createLights();

    lights.put({ animators: [], lights: [POINT] });
    lights.update(createCamera(), 0, DEFAULT_RENDERER_LIGHTS_SETTINGS, new LodUniforms(), 0);

    const record: Array<number> = readRecord(lights, 0);

    expect(lights.count).toBe(1);
    expect(lights.uniforms.count.value).toBe(1);
    expect(record.slice(0, 3)).toEqual([1, 2, -10]);
    // `1 / L_R²`, `L_R` 95% of the range.
    expect(record[3]).toBeCloseTo(1 / (4 * 0.95) ** 2, 6);
    expect(record.slice(4, 7)).toEqual([0.5, 0.25, 1]);
    expect(record[7]).toBeCloseTo(toSunSpecular([0.5, 0.25, 1]), 6);
    // No cone, no projector.
    expect(record[11]).toBe(-2);
    expect(record[19]).toBe(-1);
    // Bound by its range.
    expect(record.slice(20, 24)).toEqual([1, 2, -10, 4]);
  });

  it("leaves the level file's lights out unless asked, and whatever stands out of view", () => {
    const lights: SceneLights = createLights();
    const behind: IRendererLight = { ...POINT, position: [0, 0, 20] };
    const level: IRendererLight = { ...POINT, isLevel: true };

    lights.put({ animators: [], lights: [behind, level] });
    lights.update(createCamera(), 0, DEFAULT_RENDERER_LIGHTS_SETTINGS, new LodUniforms(), 0);
    expect(lights.count).toBe(0);

    lights.update(
      createCamera(),
      0,
      { ...DEFAULT_RENDERER_LIGHTS_SETTINGS, isLevelLights: true },
      new LodUniforms(),
      0
    );
    expect(lights.count).toBe(1);
  });

  it("animates a colour by its key times the scale, replacing its own", () => {
    const lights: SceneLights = createLights();

    lights.put({
      animators: [{ colors: [[255, 0, 51]], fps: 10, frameCount: 10, frames: [0] }],
      lights: [{ ...POINT, animator: 0, animatorScale: 2 / 255 }],
    });
    lights.update(createCamera(), 3, DEFAULT_RENDERER_LIGHTS_SETTINGS, new LodUniforms(), 0);

    const [red, green, blue] = readRecord(lights, 0).slice(4, 7);

    expect(red).toBeCloseTo(2, 6);
    expect(green).toBe(0);
    expect(blue).toBeCloseTo(0.4, 6);
  });

  it("frames a spot's projection square to its direction, through the slot its projector took", () => {
    const lights: SceneLights = createLights();
    const spot: IRendererLight = {
      ...POINT,
      cone: Math.PI / 2,
      // Down, turned with a right that is not square to it.
      direction: [0, -1, 0],
      kind: ERendererLightKind.SPOT,
      projector: "lights\\lights_spot01",
      right: [1, 0.5, 0],
    };

    lights.put({ animators: [], lights: [{ ...spot, projector: "other" }, spot] });
    lights.update(createCamera(), 0, DEFAULT_RENDERER_LIGHTS_SETTINGS, new LodUniforms(), 0);

    const record: Array<number> = readRecord(lights, 1);
    const [direction, right, up] = [record.slice(8, 11), record.slice(12, 15), record.slice(16, 19)];

    expect(lights.count).toBe(2);
    expect(direction[0]).toBeCloseTo(0, 6);
    expect(direction[1]).toBeCloseTo(-1, 6);
    expect(direction[2]).toBeCloseTo(0, 6);
    expect(record[11]).toBeCloseTo(Math.cos(Math.PI / 4), 6);
    expect(right[0]).toBeCloseTo(1, 6);
    expect(right[1]).toBeCloseTo(0, 6);
    expect(right[2]).toBeCloseTo(0, 6);
    // The engine's `up = dir x right` is `+z` in its own space, so `-z` mirrored.
    expect(up.map((it) => (Math.abs(it) < 1e-6 ? 0 : it))).toEqual([0, 0, -1]);
    expect(record[15]).toBeCloseTo(1 / Math.tan((Math.PI / 2 + (3.5 * Math.PI) / 180) / 2), 6);
    expect(record[19]).toBe(1);
  });

  it("fades a shadowed spot with its share of the screen and drops it past the far threshold", () => {
    const lights: SceneLights = createLights();
    const lod: LodUniforms = new LodUniforms();
    const spot: IRendererLight = {
      ...POINT,
      cone: Math.PI / 3,
      direction: [0, 0, -1],
      isShadowed: true,
      kind: ERendererLightKind.SPOT,
      position: [0, 0, -40],
    };

    lod.glodStart.value = 0.001;
    lod.glodEnd.value = 0.0001;
    lights.put({ animators: [], lights: [spot, { ...spot, position: [0, 0, -400] }] });
    lights.update(createCamera(), 0, DEFAULT_RENDERER_LIGHTS_SETTINGS, lod, 0);

    // The acute cone's sphere: `R / (2 cos²(c / 2))` ahead of it, at `-44.44`.
    const radius: number = 4 / (2 * Math.cos(Math.PI / 6) ** 2);
    const area: number = (0.5 * radius) / ((40 + radius) ** 2 + 0.001);
    const fade: number = Math.sqrt((area - 0.0001) / (0.001 - 0.0001));

    expect(lights.count).toBe(1);
    expect(readRecord(lights, 0)[4]).toBeCloseTo(0.5 * fade, 6);
  });
});
