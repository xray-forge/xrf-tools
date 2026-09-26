import { uniform } from "three/tsl";
import { PerspectiveCamera, Vector4 } from "three/webgpu";

/** Shadow faces drawn at most in one frame, so a level opening fills the atlas over a few frames rather than in one. */
export const LIGHT_SHADOW_FACE_BUDGET: number = 8;

/** Tiles the view is cut into across and down, and slices into along it, each a cluster the lights are binned into. */
export const LIGHT_CLUSTERS_X: number = 16;
export const LIGHT_CLUSTERS_Y: number = 9;
export const LIGHT_CLUSTERS_Z: number = 24;
export const LIGHT_CLUSTERS: number = LIGHT_CLUSTERS_X * LIGHT_CLUSTERS_Y * LIGHT_CLUSTERS_Z;

/** Lights one cluster holds at most; any more reaching it go unlit there. */
export const LIGHT_CLUSTER_CAPACITY: number = 64;

/**
 * What the lights are binned and lit by: how many stand in view this frame, and the view the clusters cut, its depth
 * sliced exponentially from the near plane to the far one.
 */
export class LightsUniforms {
  public readonly count = uniform(0, "uint");
  public readonly near = uniform(0.1);
  public readonly far = uniform(1000);
  /** The projection's `x` and `y` scales and its offsets, jitter included: `elements` 0, 5, 8 and 9. */
  public readonly projection = uniform(new Vector4(1, 1, 0, 0));

  /**
   * @param camera - The camera drawing the frame, its projection jittered as the frame draws it.
   * @param count - Lights standing in view.
   */
  public follow(camera: PerspectiveCamera, count: number): void {
    const { elements } = camera.projectionMatrix;

    this.count.value = count;
    this.near.value = camera.near;
    this.far.value = camera.far;
    this.projection.value.set(elements[0], elements[5], elements[8], elements[9]);
  }
}
