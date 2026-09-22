import { uniform } from "three/tsl";
import { Matrix4, PerspectiveCamera } from "three/webgpu";

/**
 * The drawing camera's matrices, for full screen passes whose own camera is the quad's.
 */
export class CameraUniforms {
  /** Clip to view, for positions rebuilt from depth. */
  public readonly projectionInverse = uniform(new Matrix4());
  /** View to world, the engine's `m_v2w`. */
  public readonly viewToWorld = uniform(new Matrix4());
  /** The far plane, for depth shown as a picture. */
  public readonly far = uniform(1);

  /**
   * @param camera - The camera about to draw, with its matrices current.
   */
  public follow(camera: PerspectiveCamera): void {
    this.projectionInverse.value.copy(camera.projectionMatrixInverse);
    this.viewToWorld.value.copy(camera.matrixWorld);
    this.far.value = camera.far;
  }
}
