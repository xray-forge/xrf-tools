import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { PerspectiveCamera, Vector3 } from "three/webgpu";

import { setCameraAspect, setCameraLens } from "#/camera/camera-lens";
import { isSameCameraStart, toCameraPose } from "#/camera/camera-pose";
import { bindDragCursor } from "#/camera/drag-cursor";
import { toDolliedPosition } from "#/camera/orbit-dolly";
import { IRendererCameraController } from "#/camera/renderer-camera-controller";
import { ERendererCameraController, TRendererCamera } from "#/contract/renderer-camera";
import { ERendererCameraCommand, TRendererCameraCommand } from "#/contract/renderer-camera-command";
import { IRendererCameraPose } from "#/contract/renderer-camera-pose";
import { IRendererOrbitCamera } from "#/contract/renderer-orbit-camera";
import { RenderProxyElement } from "#/input/render-proxy-element";

/** Where a camera starts before its consumer describes one: a unit of distance back from the origin. */
const DEFAULT_ORBIT_CAMERA: IRendererOrbitCamera = {
  far: 100,
  fieldOfView: 45,
  kind: ERendererCameraController.ORBIT,
  near: 0.01,
  position: [0, 0, 3],
  target: [0, 0, 0],
};

/**
 * A camera orbiting a target, turned by the gestures forwarded to the canvas's stand-in.
 */
export class OrbitCameraController implements IRendererCameraController {
  public readonly camera: PerspectiveCamera = new PerspectiveCamera();

  private readonly controls: OrbitControls;
  private readonly unbindCursor: () => void;
  private description: IRendererOrbitCamera = DEFAULT_ORBIT_CAMERA;

  public constructor(element: RenderProxyElement) {
    // Cast because three types an element it only ever listens to, measures and writes a cursor on - which is
    // exactly what a stand-in for one answers.
    this.controls = new OrbitControls(this.camera, element as unknown as HTMLElement);
    // Undamped, a drag turns the camera by exactly what the pointer moved, and stops when it stops.
    this.controls.enableDamping = false;
    this.unbindCursor = bindDragCursor(this.controls, element);
    setCameraLens(this.camera, this.description);
    this.reset();
  }

  /**
   * @param description - The camera the consumer wants, from where it starts.
   */
  public describe(description: TRendererCamera): boolean {
    if (description.kind !== ERendererCameraController.ORBIT) {
      return false;
    }

    // The same start again is a new lens, not a request to go back.
    const isMoved: boolean = !isSameCameraStart(description, this.description);

    this.description = description;
    setCameraLens(this.camera, description);

    if (isMoved) {
      this.reset();
    }

    return isMoved;
  }

  /**
   * @param command - What to do with the camera.
   */
  public command(command: TRendererCameraCommand): void {
    switch (command.kind) {
      case ERendererCameraCommand.RESET:
        return this.reset();

      case ERendererCameraCommand.DOLLY: {
        const { x, y, z } = this.camera.position;
        const target: Vector3 = this.controls.target;

        this.camera.position.set(
          ...toDolliedPosition(
            [x, y, z],
            [target.x, target.y, target.z],
            command.step,
            this.controls.minDistance,
            this.controls.maxDistance
          )
        );
        this.controls.update();

        return;
      }
    }
  }

  /**
   * @param width - Drawing width, in any unit the height shares.
   * @param height - Drawing height.
   */
  public resize(width: number, height: number): void {
    setCameraAspect(this.camera, width, height);
  }

  /** Applies what the gestures moved since the frame before. */
  public update(): void {
    this.controls.update();
  }

  /**
   * @returns Where the camera is and what it looks at.
   */
  public get pose(): IRendererCameraPose {
    return toCameraPose(this.camera.position, this.controls.target);
  }

  public dispose(): void {
    this.unbindCursor();
    this.controls.dispose();
  }

  private reset(): void {
    this.camera.position.set(...this.description.position);
    this.controls.target.set(...this.description.target);
    this.controls.update();
  }
}
