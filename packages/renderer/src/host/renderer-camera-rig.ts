import { PerspectiveCamera } from "three/webgpu";

import { IRendererCameraController } from "#/camera/camera-controller";
import { FlyCameraController } from "#/camera/fly-camera-controller";
import { OrbitCameraController } from "#/camera/orbit-camera-controller";
import {
  ERendererCameraController,
  IRendererCameraPose,
  TRendererCamera,
  TRendererCameraCommand,
} from "#/contract/renderer-camera";
import { RenderProxyElement } from "#/input/render-proxy-element";

/** How each kind of camera a consumer can ask for is driven. */
const CAMERA_CONTROLLERS: Record<
  ERendererCameraController,
  (element: RenderProxyElement) => IRendererCameraController
> = {
  [ERendererCameraController.FLY]: (element) => new FlyCameraController(element),
  [ERendererCameraController.ORBIT]: (element) => new OrbitCameraController(element),
};

/**
 * The drawing camera and whichever controller drives it: an orbit until the consumer asks for another kind.
 */
export class RendererCameraRig {
  private readonly element: RenderProxyElement;
  private controller: IRendererCameraController;
  private kind: ERendererCameraController = ERendererCameraController.ORBIT;
  /** The view's size, for a controller made after the view was measured. */
  private width: number = 1;
  private height: number = 1;

  /**
   * @param element - What stands in for the canvas the controllers listen to.
   */
  public constructor(element: RenderProxyElement) {
    this.element = element;
    this.controller = CAMERA_CONTROLLERS[this.kind](element);
  }

  public get camera(): PerspectiveCamera {
    return this.controller.camera;
  }

  public get pose(): IRendererCameraPose {
    return this.controller.pose;
  }

  /**
   * Describes the camera to the controller its kind asks for, replacing the one driving it for another kind.
   *
   * @param camera - The camera the consumer wants.
   */
  public describe(camera: TRendererCamera): void {
    if (camera.kind !== this.kind) {
      this.controller.dispose();
      this.controller = CAMERA_CONTROLLERS[camera.kind](this.element);
      this.controller.resize(this.width, this.height);
      this.kind = camera.kind;
    }

    this.controller.describe(camera);
  }

  /**
   * @param command - What to do with the camera.
   */
  public command(command: TRendererCameraCommand): void {
    this.controller.command(command);
  }

  /**
   * @param width - Drawing width, in css pixels.
   * @param height - Drawing height, in css pixels.
   */
  public resize(width: number, height: number): void {
    this.width = width;
    this.height = height;
    this.controller.resize(width, height);
  }

  /**
   * Advances the camera and brings its matrices up to date for the frame about to be drawn.
   *
   * @param delta - Seconds since the last frame.
   */
  public update(delta: number): void {
    this.controller.update(delta);
    this.controller.camera.updateMatrixWorld();
  }

  public dispose(): void {
    this.controller.dispose();
  }
}
