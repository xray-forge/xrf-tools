import { Nullable } from "@xrf/types";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { PerspectiveCamera } from "three/webgpu";

import { IRendererCameraController } from "#/camera/camera-controller";
import { bindDragCursor } from "#/camera/drag-cursor";
import { toDolliedPosition } from "#/camera/orbit-dolly";
import {
  ERendererCameraCommand,
  ERendererCameraController,
  IRendererCameraPose,
  IRendererOrbitCamera,
  TRendererCamera,
  TRendererCameraCommand,
} from "#/contract/renderer-camera";
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
  private aspect: Nullable<number> = null;

  public constructor(element: RenderProxyElement) {
    // Cast because three types an element it only ever listens to, measures and writes a cursor on - which is
    // exactly what a stand-in for one answers.
    this.controls = new OrbitControls(this.camera, element as unknown as HTMLElement);
    this.controls.enableDamping = true;
    this.unbindCursor = bindDragCursor(this.controls, element);
    this.describe(DEFAULT_ORBIT_CAMERA);
  }

  /**
   * @param description - The camera the consumer wants, from where it starts.
   */
  public describe(description: TRendererCamera): void {
    if (description.kind !== ERendererCameraController.ORBIT) {
      return;
    }

    this.description = description;
    this.camera.fov = description.fieldOfView;
    this.camera.near = description.near;
    this.camera.far = description.far;
    this.camera.updateProjectionMatrix();
    this.reset();
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
        const target = this.controls.target;

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
    const aspect: number = width / Math.max(height, 1);

    if (aspect !== this.aspect) {
      this.aspect = aspect;
      this.camera.aspect = aspect;
      this.camera.updateProjectionMatrix();
    }
  }

  /** Advances the damping, once a frame. */
  public update(): void {
    this.controls.update();
  }

  /**
   * @returns Where the camera is and what it looks at.
   */
  public get pose(): IRendererCameraPose {
    const { position } = this.camera;
    const { target } = this.controls;

    return { position: [position.x, position.y, position.z], target: [target.x, target.y, target.z] };
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
