import { Camera, Frustum, Matrix4, Plane, Vector4 } from "three/webgpu";

/** What a camera's view and projection are multiplied into, for the frustum they make. */
const VIEW_PROJECTION: Matrix4 = new Matrix4();

/**
 * @param camera - A camera, its matrices current.
 * @param out - Where its frustum is written.
 * @returns Its frustum, its depth taken as the camera takes it, reversed or not.
 */
export function toCameraFrustum(camera: Camera, out: Frustum): Frustum {
  VIEW_PROJECTION.multiplyMatrices(camera.projectionMatrix, camera.matrixWorldInverse);

  return out.setFromProjectionMatrix(VIEW_PROJECTION, camera.coordinateSystem, camera.reversedDepth);
}

/**
 * @param planes - A frustum's planes.
 * @param out - A vector a plane, written its normal, pointing in, then its constant, as the shaders test by.
 */
export function toPlaneVectors(planes: ReadonlyArray<Plane>, out: ReadonlyArray<Vector4>): void {
  planes.forEach(({ normal, constant }: Plane, index: number) =>
    out[index].set(normal.x, normal.y, normal.z, constant)
  );
}
