import { describe, expect, it } from "@jest/globals";
import { Matrix4, Object3D, PerspectiveCamera } from "three/webgpu";

import { MotionUniforms } from "#/uniforms/motion-uniforms";

/** What three hands an object update callback. */
interface IObjectUpdate {
  update(frame: { object: Object3D }): void;
}

describe("MotionUniforms", () => {
  it("measures against itself on the first frame, then against the frame before", () => {
    const uniforms: MotionUniforms = new MotionUniforms();
    const camera: PerspectiveCamera = new PerspectiveCamera(60, 1, 0.2, 100);

    camera.updateMatrixWorld();
    uniforms.follow(camera);

    const first: Matrix4 = uniforms.viewProjection.value.clone();

    expect(uniforms.previousViewProjection.value.equals(first)).toBe(true);

    camera.position.set(1, 0, 0);
    camera.updateMatrixWorld();
    uniforms.follow(camera);

    expect(uniforms.previousViewProjection.value.equals(first)).toBe(true);
    expect(uniforms.viewProjection.value.equals(first)).toBe(false);
    expect(uniforms.previousView.value.elements[12]).toBeCloseTo(0, 10);
  });

  it("gives a plain object the matrix it drew with the frame before, however often it draws in a frame", () => {
    const uniforms: MotionUniforms = new MotionUniforms();
    const camera: PerspectiveCamera = new PerspectiveCamera();
    const object: Object3D = new Object3D();
    const node = uniforms.previousModelWorld as unknown as IObjectUpdate;

    uniforms.follow(camera);
    object.updateMatrixWorld();
    node.update({ object });

    expect(uniforms.previousModelWorld.value.equals(new Matrix4())).toBe(true);

    uniforms.follow(camera);
    object.position.set(0, 3, 0);
    object.updateMatrixWorld();
    node.update({ object });
    node.update({ object });

    expect(uniforms.previousModelWorld.value.elements[13]).toBe(0);

    uniforms.follow(camera);
    node.update({ object });

    expect(uniforms.previousModelWorld.value.elements[13]).toBe(3);
  });
});
