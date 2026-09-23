import { PerspectiveCamera } from "three/webgpu";

import { IRendererCameraPose, TRendererCamera, TRendererCameraCommand } from "#/contract/renderer-camera";

/**
 * Drives the drawing camera from what the consumer described and what the person does to the canvas.
 */
export interface IRendererCameraController {
  readonly camera: PerspectiveCamera;

  /** Where the camera is and what it looks at. */
  readonly pose: IRendererCameraPose;

  /**
   * @param description - The camera the consumer wants, of this controller's kind.
   */
  describe(description: TRendererCamera): void;

  /**
   * @param command - What to do with the camera.
   */
  command(command: TRendererCameraCommand): void;

  /**
   * @param width - Drawing width, in any unit the height shares.
   * @param height - Drawing height.
   */
  resize(width: number, height: number): void;

  /**
   * Advances the camera, once a frame.
   *
   * @param delta - Seconds since the last frame.
   */
  update(delta: number): void;

  dispose(): void;
}
