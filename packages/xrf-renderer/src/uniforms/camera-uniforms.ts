import { renderGroup, uniform } from "three/tsl";
import { Matrix4, PerspectiveCamera, UniformNode } from "three/webgpu";

/**
 * The drawing camera's matrices, for full screen passes whose own camera is the quad's.
 */
export class CameraUniforms {
  /** Clip to view, for positions rebuilt from depth. */
  public readonly projectionInverse: UniformNode<"mat4", Matrix4> = uniform(new Matrix4()).setGroup(renderGroup);
  /** View to world, the engine's `m_v2w`. */
  public readonly viewToWorld: UniformNode<"mat4", Matrix4> = uniform(new Matrix4()).setGroup(renderGroup);
  /** The far plane, for depth shown as a picture. */
  public readonly far: UniformNode<"float", number> = uniform(1).setGroup(renderGroup);

  /**
   * @param camera - The camera about to draw, with its matrices current.
   */
  public follow(camera: PerspectiveCamera): void {
    this.projectionInverse.value.copy(camera.projectionMatrixInverse);
    this.viewToWorld.value.copy(camera.matrixWorld);
    this.far.value = camera.far;
  }
}
