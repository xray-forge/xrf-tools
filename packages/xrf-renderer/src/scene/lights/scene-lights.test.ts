import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera } from "three/webgpu";

import { EMPTY_RENDERER_LIGHTS_REPORT } from "#/contract/renderer-lights-report";
import { DEFAULT_RENDERER_LIGHTS_SETTINGS, IRendererLightsSettings } from "#/contract/renderer-lights-settings";
import { ERendererLightKind, TRendererLight } from "#/contract/scene/renderer-light";
import { IRendererPointLight } from "#/contract/scene/renderer-point-light";
import { IRendererSpotLight } from "#/contract/scene/renderer-spot-light";
import { adoptRendererConventions } from "#/internals/camera-conventions";
import { toSunSpecular } from "#/lighting/base-lighting";
import { LIGHT_NO_CONE, LIGHT_RECORD, LIGHT_VECTORS, MAX_LIGHTS } from "#/scene/lights/light-record";
import { SceneLights } from "#/scene/lights/scene-lights";
import { ISceneLightsFrame } from "#/scene/lights/scene-lights-frame";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";
import { RendererTextures } from "#/texture/renderer-textures";
import { LodUniforms } from "#/uniforms/lod-uniforms";

const POINT: IRendererPointLight = {
  animatorScale: 0,
  color: [0.5, 0.25, 1],
  isLevel: false,
  isShadowed: false,
  kind: ERendererLightKind.POINT,
  near: 0,
  position: [1, 2, -10],
  range: 4,
};

const SPOT: IRendererSpotLight = {
  ...POINT,
  cone: Math.PI / 2,
  direction: [0, -1, 0],
  kind: ERendererLightKind.SPOT,
  right: [1, 0, 0],
};

/** A camera at the origin looking down `-z`, as a renderer's camera is before it moves. */
function createCamera(): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 0.1, 1000);

  adoptRendererConventions(camera);
  camera.updateMatrixWorld();

  return camera;
}

function createLights(random?: () => number): SceneLights {
  return new SceneLights(
    new RendererTextures(
      () => {},
      () => {}
    ),
    new StaticShadowChanges(),
    random
  );
}

/** A frame seen from the origin, the view and the drawing one camera. */
function createFrame(part: Partial<ISceneLightsFrame> = {}): ISceneLightsFrame {
  const camera: PerspectiveCamera = createCamera();

  return {
    camera,
    isWindy: false,
    lod: new LodUniforms(),
    settings: DEFAULT_RENDERER_LIGHTS_SETTINGS,
    time: 0,
    view: camera,
    ...part,
  };
}

/** One vector of a light's record. */
function readVector(lights: SceneLights, light: number, vector: number): Array<number> {
  const at: number = (light * LIGHT_VECTORS + vector) * 4;

  return Array.from((lights.records.buffer.array as Float32Array).subarray(at, at + 4));
}

describe("SceneLights", () => {
  it("writes a point in view space with the engine's falloff, colour and specular weight", () => {
    const lights: SceneLights = createLights();

    lights.put({ animators: [], lights: [POINT] });
    lights.update(createFrame());

    const [x, y, z, falloff] = readVector(lights, 0, LIGHT_RECORD.position);

    expect(lights.count).toBe(1);
    expect(lights.clusters.uniforms.count.value).toBe(1);
    expect([x, y, z]).toEqual([1, 2, -10]);
    // `1 / L_R²`, `L_R` 95% of the range.
    expect(falloff).toBeCloseTo(1 / (4 * 0.95) ** 2, 6);
    expect(readVector(lights, 0, LIGHT_RECORD.color).slice(0, 3)).toEqual([0.5, 0.25, 1]);
    expect(readVector(lights, 0, LIGHT_RECORD.color)[3]).toBeCloseTo(toSunSpecular([0.5, 0.25, 1]), 6);
    // No cone, no projector, no shadow.
    expect(readVector(lights, 0, LIGHT_RECORD.axis)[3]).toBe(LIGHT_NO_CONE);
    expect(readVector(lights, 0, LIGHT_RECORD.up)[3]).toBe(-1);
    expect(readVector(lights, 0, LIGHT_RECORD.shadow)).toEqual([0, 0, 0, 0]);
    expect(readVector(lights, 0, LIGHT_RECORD.sphere)).toEqual([1, 2, -10, 4]);
  });

  it("leaves the level file's lights out unless asked, whatever stands out of view, and all while the lights are off", () => {
    const lights: SceneLights = createLights();
    const settings: IRendererLightsSettings = { ...DEFAULT_RENDERER_LIGHTS_SETTINGS, isLevelLights: true };

    lights.put({
      animators: [],
      lights: [
        { ...POINT, position: [0, 0, 20] },
        { ...POINT, isLevel: true },
      ],
    });
    lights.update(createFrame());
    expect(lights.count).toBe(0);

    lights.update(createFrame({ settings }));
    expect(lights.count).toBe(1);

    lights.update(createFrame({ settings: { ...settings, isEnabled: false } }));
    expect(lights.count).toBe(0);
    expect(lights.clusters.uniforms.count.value).toBe(0);
  });

  it("animates a colour by its key times the scale, replacing its own", () => {
    const lights: SceneLights = createLights();

    lights.put({
      animators: [{ colors: [[255, 0, 51]], fps: 10, frameCount: 10, frames: [0] }],
      lights: [{ ...POINT, animator: 0, animatorScale: 2 / 255 }],
    });
    lights.update(createFrame({ time: 3 }));

    const [red, green, blue] = readVector(lights, 0, LIGHT_RECORD.color);

    expect(red).toBeCloseTo(2, 6);
    expect(green).toBe(0);
    expect(blue).toBeCloseTo(0.4, 6);
  });

  it("strays a zone's range by its jitter, as the random source says", () => {
    const lights: SceneLights = createLights(() => 1);

    lights.put({ animators: [], lights: [{ ...POINT, rangeJitter: 1 }] });
    lights.update(createFrame());

    expect(readVector(lights, 0, LIGHT_RECORD.position)[3]).toBeCloseTo(1 / (5 * 0.95) ** 2, 6);
    // Bound as far as the range strays either way.
    expect(readVector(lights, 0, LIGHT_RECORD.sphere)[3]).toBe(5);
  });

  it("frames a spot's projection square to its direction, through the slot its projector took", () => {
    const lights: SceneLights = createLights();
    // Down, turned with a right that is not square to it.
    const spot: IRendererSpotLight = { ...SPOT, projector: "lights\\lights_spot01", right: [1, 0.5, 0] };

    lights.put({ animators: [], lights: [{ ...spot, projector: "other" }, spot] });
    lights.update(createFrame());

    const [direction, right, up] = [LIGHT_RECORD.axis, LIGHT_RECORD.right, LIGHT_RECORD.up].map((vector: number) =>
      readVector(lights, 1, vector)
    );

    expect(lights.count).toBe(2);
    expect(direction[0]).toBeCloseTo(0, 6);
    expect(direction[1]).toBeCloseTo(-1, 6);
    expect(direction[2]).toBeCloseTo(0, 6);
    expect(direction[3]).toBeCloseTo(Math.cos(Math.PI / 4), 6);
    expect(right[0]).toBeCloseTo(1, 6);
    expect(right[1]).toBeCloseTo(0, 6);
    expect(right[2]).toBeCloseTo(0, 6);
    // The engine's `up = dir x right` is `+z` in its own space, so `-z` mirrored.
    expect(up.slice(0, 3).map((it: number) => (Math.abs(it) < 1e-6 ? 0 : it))).toEqual([0, 0, -1]);
    expect(right[3]).toBeCloseTo(1 / Math.tan((Math.PI / 2 + (3.5 * Math.PI) / 180) / 2), 6);
    expect(up[3]).toBe(1);
  });

  it("fades a shadowed light of either kind with its share of the screen, and drops it past the far threshold", () => {
    const lights: SceneLights = createLights();
    const lod: LodUniforms = new LodUniforms();
    const spot: IRendererSpotLight = {
      ...SPOT,
      cone: Math.PI / 3,
      direction: [0, 0, -1],
      isShadowed: true,
      position: [0, 0, -40],
    };
    const point: IRendererPointLight = { ...POINT, isShadowed: true, position: [0, 0, -60] };

    lod.glodStart.value = 0.001;
    lod.glodEnd.value = 0.0001;
    lights.put({ animators: [], lights: [spot, { ...spot, position: [0, 0, -400] }] });
    lights.update(createFrame({ lod }));

    // The acute cone's sphere: `R / (2 cos²(c / 2))` ahead of it, at `-44.44`.
    const radius: number = 4 / (2 * Math.cos(Math.PI / 6) ** 2);
    const area: number = (0.5 * radius) / ((40 + radius) ** 2 + 0.00001);

    expect(lights.count).toBe(1);
    expect(readVector(lights, 0, LIGHT_RECORD.color)[0]).toBeCloseTo(0.5 * Math.sqrt((area - 0.0001) / 0.0009), 6);

    lights.put({ animators: [], lights: [point] });
    lights.update(createFrame({ lod }));
    lights.shadows.markDrawn();
    lights.update(createFrame({ lod }));

    // A point fades face by face, each omni part's sphere its range over root two across, that far along the face: the
    // one facing the camera nearer, the one facing away farther; its colour stays whole.
    const part: number = 4 * Math.SQRT1_2;

    function toFaceFade(distance: number): number {
      return Math.sqrt(((0.5 * part) / (distance ** 2 + 0.00001) - 0.0001) / 0.0009);
    }

    expect(readVector(lights, 0, LIGHT_RECORD.color)[0]).toBeCloseTo(0.5, 6);
    expect(readVector(lights, 0, LIGHT_RECORD.faces + 4)[3]).toBeCloseTo(toFaceFade(60 - part), 5);
    expect(readVector(lights, 0, LIGHT_RECORD.faces + 5)[3]).toBeCloseTo(toFaceFade(60 + part), 5);
  });

  it("lights a shadowed light only once its faces are drawn, with its faces' squares", () => {
    const lights: SceneLights = createLights();
    const shadowed: Array<TRendererLight> = [
      { ...POINT, isShadowed: true },
      { ...POINT, isShadowed: true, position: [-1, 2, -12] },
    ];

    lights.put({ animators: [], lights: shadowed });
    // Twelve faces, eight drawn a frame: the second light waits.
    lights.update(createFrame());

    expect(lights.count).toBe(1);
    expect(lights.shadowed).toBe(1);
    expect(readVector(lights, 0, LIGHT_RECORD.shadow)[2]).toBe(6);

    lights.shadows.markDrawn();
    lights.update(createFrame());

    expect(lights.count).toBe(2);
    expect(lights.report.shadowed).toBe(2);
  });

  it("draws every face again once the lights' shadows are back on, the atlas having gone with them", () => {
    const lights: SceneLights = createLights();
    const unshadowed: IRendererLightsSettings = { ...DEFAULT_RENDERER_LIGHTS_SETTINGS, isShadowed: false };

    lights.put({ animators: [], lights: [{ ...POINT, isShadowed: true }] });
    lights.update(createFrame());
    lights.shadows.markDrawn();
    lights.update(createFrame());

    expect(lights.shadows.queue).toHaveLength(0);

    lights.update(createFrame({ settings: unshadowed }));
    lights.update(createFrame());

    // Lit this frame from its six faces drawn again in it, not from the atlas the shadows had before.
    expect(lights.shadows.queue).toHaveLength(6);
    expect(lights.shadowed).toBe(1);
  });

  it("keeps the nearest lights the records hold, and counts the rest", () => {
    const lights: SceneLights = createLights();
    const crowd: Array<TRendererLight> = Array.from({ length: MAX_LIGHTS + 6 }, (_, index: number) => ({
      ...POINT,
      position: [0, 0, -10 - (MAX_LIGHTS + 6 - index) * 0.1],
      range: 1,
    }));

    lights.put({ animators: [], lights: crowd });
    lights.update(createFrame());

    expect(lights.count).toBe(MAX_LIGHTS);
    expect(lights.report.excessLights).toBe(6);
    // The nearest first: the last put, standing nearest.
    expect(readVector(lights, 0, LIGHT_RECORD.position)[2]).toBeCloseTo(-10.1, 4);
  });

  it("leaves the record of a shadowed light waiting for its faces to the next light in view", () => {
    const lights: SceneLights = createLights();
    // Two shadowed points nearest, twelve faces at eight a frame, then as many as fill the records past them.
    const crowd: Array<TRendererLight> = [
      { ...POINT, isShadowed: true, position: [0, 0, -5] },
      { ...POINT, isShadowed: true, position: [0, 0, -6] },
      ...Array.from({ length: MAX_LIGHTS - 1 }, (_: unknown, index: number): IRendererPointLight => ({
        ...POINT,
        position: [0, 0, -10 - index * 0.1],
        range: 1,
      })),
    ];

    lights.put({ animators: [], lights: crowd });
    lights.update(createFrame());

    expect(lights.count).toBe(MAX_LIGHTS);
    expect(lights.shadowed).toBe(1);
    expect(lights.report.excessLights).toBe(0);
  });

  it("reports nothing once the lights are let go", () => {
    const lights: SceneLights = createLights();

    lights.put({ animators: [], lights: [POINT] });
    lights.update(createFrame());
    lights.release();

    expect({ ...lights.report, atlas: EMPTY_RENDERER_LIGHTS_REPORT.atlas }).toEqual(EMPTY_RENDERER_LIGHTS_REPORT);
  });
});
