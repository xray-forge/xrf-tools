import { describe, expect, it } from "@jest/globals";
import { Matrix4, PerspectiveCamera, Vector3, Vector4, WebGPUCoordinateSystem } from "three/webgpu";

import { DEFAULT_RENDERER_SHADOW_SETTINGS } from "#/contract/renderer-features";
import { ShadowUniforms } from "#/uniforms/shadow-uniforms";

function createCamera(): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(67.5, 1.7, 0.2, 5000);

  camera.coordinateSystem = WebGPUCoordinateSystem;
  camera.position.set(0, 10, 0);
  camera.lookAt(0, 10, 1);
  camera.updateMatrixWorld(true);

  return camera;
}

describe("ShadowUniforms", () => {
  it("draws as many cascades as the settings widths, and none while shadows are off", () => {
    const shadows: ShadowUniforms = new ShadowUniforms();

    shadows.fit(createCamera(), new Vector3(0, -1, 0), DEFAULT_RENDERER_SHADOW_SETTINGS);

    expect(shadows.drawn).toBe(3);

    shadows.fit(createCamera(), new Vector3(0, -1, 0), { ...DEFAULT_RENDERER_SHADOW_SETTINGS, isEnabled: false });

    expect(shadows.drawn).toBe(0);
  });

  // The sun reads what a map holds: a cascade fitted but not drawn again keeps the matrix it was drawn with.
  it("hands the sun a cascade's matrix only once its map is drawn", () => {
    const shadows: ShadowUniforms = new ShadowUniforms();

    shadows.fit(createCamera(), new Vector3(0, -1, 0), DEFAULT_RENDERER_SHADOW_SETTINGS);

    expect(shadows.matrices[1].value.equals(new Matrix4())).toBe(true);

    shadows.commit(1);

    const cascade = shadows.cascades[1].camera;

    expect(
      shadows.matrices[1].value.equals(
        new Matrix4().multiplyMatrices(cascade.projectionMatrix, cascade.matrixWorldInverse)
      )
    ).toBe(true);
    expect((shadows.texels.value as Vector4).y).toBeCloseTo(40 / 2048);
  });
});
