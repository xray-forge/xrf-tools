import { MathUtils, PerspectiveCamera, Vector3 } from "three/webgpu";

/** One edge of the view: where it starts and which way it runs, normalized. */
export interface ISunViewRay {
  readonly origin: Vector3;
  readonly direction: Vector3;
}

/** A corner of the near plane, in the camera's own space, reused. */
const CORNER: Vector3 = new Vector3();

/**
 * The view's four edges, which the sun's cascades are placed along one after another: each cascade starts where the
 * edges leave the one before it (`render_phase_sun.cpp`, the `rays` each cascade hands the next).
 */
export class SunViewRays {
  public readonly rays: ReadonlyArray<ISunViewRay> = Array.from({ length: 4 }, () => ({
    direction: new Vector3(),
    origin: new Vector3(),
  }));
  /** The same edges as they start, at the near plane, which every cascade holds the first stretch of. */
  public readonly near: ReadonlyArray<ISunViewRay> = Array.from({ length: 4 }, () => ({
    direction: new Vector3(),
    origin: new Vector3(),
  }));

  /**
   * Starts the edges over at the camera's near plane, as the first cascade takes them.
   *
   * @param camera - The camera drawing, its world matrix current.
   */
  public reset(camera: PerspectiveCamera): void {
    const y: number = Math.tan(MathUtils.degToRad(camera.fov) / 2);
    const x: number = y * camera.aspect;

    this.rays.forEach((ray: ISunViewRay, index: number) => {
      CORNER.set(index & 1 ? x : -x, index & 2 ? y : -y, -1);
      ray.direction.copy(CORNER).transformDirection(camera.matrixWorld);
      ray.origin.copy(CORNER.multiplyScalar(camera.near)).applyMatrix4(camera.matrixWorld);
      this.near[index].direction.copy(ray.direction);
      this.near[index].origin.copy(ray.origin);
    });
  }
}
