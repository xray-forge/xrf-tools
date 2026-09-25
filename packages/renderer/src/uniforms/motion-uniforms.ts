import { renderGroup, uniform } from "three/tsl";
import { Matrix4, Object3D, PerspectiveCamera, Vector2 } from "three/webgpu";

/** Where an object stood in the frame it was last drawn in, and the one before. */
interface IObjectPlacement {
  frame: number;
  current: Matrix4;
  previous: Matrix4;
}

/**
 * What the motion every G-buffer surface writes is measured with: the camera's view and projection this frame and the
 * last, both without the jitter, so motion is what moved and never the sample offset; each drawn object's matrix the
 * frame before; and the jitter itself, for the resolve to know where each sample was taken.
 */
export class MotionUniforms {
  /** World to clip, this frame, unjittered. */
  public readonly viewProjection = uniform(new Matrix4()).setGroup(renderGroup);
  /** World to clip, the frame before, unjittered. */
  public readonly previousViewProjection = uniform(new Matrix4()).setGroup(renderGroup);
  /** World to view, the frame before: how far a point stood from the camera then. */
  public readonly previousView = uniform(new Matrix4()).setGroup(renderGroup);
  /** Where this frame's samples sit off each pixel's centre, in its pixels, `y` down. */
  public readonly jitter = uniform(new Vector2()).setGroup(renderGroup);
  /** A plain object's world matrix the frame before, read per object as it draws. */
  public readonly previousModelWorld = uniform(new Matrix4()).onObjectUpdate(({ object }) =>
    this.toPreviousMatrix(object as Object3D)
  );

  private readonly placements: WeakMap<Object3D, IObjectPlacement> = new WeakMap();
  private readonly view: Matrix4 = new Matrix4();
  private frame: number = 0;

  /**
   * Takes the camera's matrices for the frame about to draw, the last frame's becoming the previous ones.
   *
   * @param camera - The drawing camera, its matrices current and not yet jittered.
   */
  public follow(camera: PerspectiveCamera): void {
    const isFirst: boolean = this.frame === 0;

    this.frame += 1;
    this.previousViewProjection.value.copy(this.viewProjection.value);
    this.previousView.value.copy(this.view);
    this.view.copy(camera.matrixWorld).invert();
    this.viewProjection.value.multiplyMatrices(camera.projectionMatrix, this.view);

    if (isFirst) {
      this.previousViewProjection.value.copy(this.viewProjection.value);
      this.previousView.value.copy(this.view);
    }
  }

  /** The object's matrix in the frame before the one drawing, its own the first time it draws. */
  private toPreviousMatrix(object: Object3D): Matrix4 {
    let placement: IObjectPlacement | undefined = this.placements.get(object);

    if (!placement) {
      placement = { current: object.matrixWorld.clone(), frame: this.frame, previous: object.matrixWorld.clone() };
      this.placements.set(object, placement);
    } else if (placement.frame !== this.frame) {
      placement.previous.copy(placement.current);
      placement.current.copy(object.matrixWorld);
      placement.frame = this.frame;
    }

    return placement.previous;
  }
}
