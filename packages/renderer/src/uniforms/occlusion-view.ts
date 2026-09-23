import { uniform } from "three/tsl";
import { Matrix4, PerspectiveCamera } from "three/webgpu";

/** One view an occlusion test projects by: the camera's view and projection, and its near plane. */
export class OcclusionView {
  public readonly view = uniform(new Matrix4());
  public readonly projection = uniform(new Matrix4());
  public readonly near = uniform(0.1);
  /** One once a view has been taken, zero before: a test against no view occludes nothing. */
  public readonly isTaken = uniform(0);

  /**
   * @param camera - The camera, with its matrices current.
   */
  public take(camera: PerspectiveCamera): void {
    this.view.value.copy(camera.matrixWorldInverse);
    this.projection.value.copy(camera.projectionMatrix);
    this.near.value = camera.near;
    this.isTaken.value = 1;
  }

  /**
   * @param view - The view to take the place of.
   */
  public copy(view: OcclusionView): void {
    this.view.value.copy(view.view.value);
    this.projection.value.copy(view.projection.value);
    this.near.value = view.near.value;
    this.isTaken.value = view.isTaken.value;
  }

  public forget(): void {
    this.isTaken.value = 0;
  }
}
