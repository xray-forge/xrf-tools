import { Maybe, Nullable } from "@xrf/types";
import { Euler, PerspectiveCamera, Vector3 } from "three/webgpu";

import { IRendererCameraController } from "#/camera/camera-controller";
import { setCameraAspect, setCameraLens } from "#/camera/camera-lens";
import { isSameCameraStart, toCameraPose } from "#/camera/camera-pose";
import { DragCursor } from "#/camera/drag-cursor";
import { EFlyKey, getFlyKey } from "#/camera/fly-keys";
import { ERenderInput } from "#/contract/render-input";
import { ERendererCameraController, TRendererCamera } from "#/contract/renderer-camera";
import { ERendererCameraCommand, TRendererCameraCommand } from "#/contract/renderer-camera-command";
import { IRendererCameraPose } from "#/contract/renderer-camera-pose";
import { IRendererFlyCamera } from "#/contract/renderer-fly-camera";
import { RenderProxyElement } from "#/input/render-proxy-element";
import { IRenderProxyEvent } from "#/input/render-proxy-event";

/** Just short of straight up, so looking at the sky never flips the horizon over. */
const MAX_PITCH: number = Math.PI / 2 - 0.001;

/**
 * The longest step one frame moves by, so a frame after a stall, a hidden window's, does not throw the camera across
 * the level. Any frame faster than four a second moves its whole time's worth.
 */
const MAX_DELTA: number = 0.25;

/** Where a camera starts before its consumer describes one. */
const DEFAULT_FLY_CAMERA: IRendererFlyCamera = {
  boost: 4,
  far: 10_000,
  fieldOfView: 67.5,
  kind: ERendererCameraController.FLY,
  near: 0.2,
  position: [0, 2, 0],
  sensitivity: 0.003,
  speed: 10,
  target: [0, 2, -1],
};

/**
 * A free camera: a drag turns it, the keys move it along where it faces, rising along its own up as it is pitched.
 * It holds yaw and pitch itself, since a rotation read back off the camera cannot tell `+π` from `-π`.
 */
export class FlyCameraController implements IRendererCameraController {
  public readonly camera: PerspectiveCamera = new PerspectiveCamera();

  private readonly element: RenderProxyElement;
  private readonly held: Set<EFlyKey> = new Set();
  private readonly euler: Euler = new Euler(0, 0, 0, "YXZ");
  private readonly ahead: Vector3 = new Vector3();
  private readonly across: Vector3 = new Vector3();
  private readonly above: Vector3 = new Vector3();

  private description: IRendererFlyCamera = DEFAULT_FLY_CAMERA;
  private yaw: number = 0;
  private pitch: number = 0;
  /** Pointer movement gathered since the last frame, which is what applies it. */
  private lookX: number = 0;
  private lookY: number = 0;
  /** The pointer dragging and where it last was, or null while nothing drags. */
  private dragged: Nullable<{ id: number; x: number; y: number }> = null;
  private readonly cursor: DragCursor;

  public constructor(element: RenderProxyElement) {
    this.element = element;
    this.cursor = new DragCursor(element);

    for (const [type, listener] of Object.entries(this.listeners)) {
      element.addEventListener(type, listener);
    }

    setCameraLens(this.camera, this.description);

    this.reset();
  }

  public describe(description: TRendererCamera): boolean {
    if (description.kind !== ERendererCameraController.FLY) {
      return false;
    }

    // The same start again is new speeds or a new lens, not a request to go back.
    const isMoved: boolean = !isSameCameraStart(description, this.description);

    this.description = description;
    setCameraLens(this.camera, description);

    if (isMoved) {
      this.reset();
    }

    return isMoved;
  }

  public command(command: TRendererCameraCommand): void {
    if (command.kind === ERendererCameraCommand.RESET) {
      this.reset();
    }
  }

  public resize(width: number, height: number): void {
    setCameraAspect(this.camera, width, height);
  }

  public update(delta: number): void {
    const { sensitivity, speed, boost } = this.description;

    // Looking before moving, because where the camera walks is where it faces.
    this.yaw -= this.lookX * sensitivity;
    this.pitch = Math.max(-MAX_PITCH, Math.min(MAX_PITCH, this.pitch - this.lookY * sensitivity));
    this.lookX = 0;
    this.lookY = 0;
    this.camera.quaternion.setFromEuler(this.euler.set(this.pitch, this.yaw, 0));

    const forward: number = this.axis(EFlyKey.FORWARD, EFlyKey.BACK);
    const strafe: number = this.axis(EFlyKey.RIGHT, EFlyKey.LEFT);
    const rise: number = this.axis(EFlyKey.UP, EFlyKey.DOWN);

    if (!forward && !strafe && !rise) {
      return;
    }

    const distance: number = speed * (this.held.has(EFlyKey.FAST) ? boost : 1) * Math.min(delta, MAX_DELTA);

    this.ahead.set(0, 0, -1).applyQuaternion(this.camera.quaternion);
    this.across.set(1, 0, 0).applyQuaternion(this.camera.quaternion);
    this.above.set(0, 1, 0).applyQuaternion(this.camera.quaternion);
    this.camera.position
      .addScaledVector(this.ahead, forward * distance)
      .addScaledVector(this.across, strafe * distance)
      .addScaledVector(this.above, rise * distance);
  }

  public get pose(): IRendererCameraPose {
    const { position, quaternion } = this.camera;

    return toCameraPose(position, new Vector3(0, 0, -1).applyQuaternion(quaternion).add(position));
  }

  public dispose(): void {
    for (const [type, listener] of Object.entries(this.listeners)) {
      this.element.removeEventListener(type, listener);
    }

    this.cursor.end();
  }

  /** Back to the start, looking at its target. */
  private reset(): void {
    const [x, y, z] = this.description.position;
    const [tx, ty, tz] = this.description.target;
    const direction: Vector3 = new Vector3(tx - x, ty - y, tz - z);

    this.camera.position.set(x, y, z);

    // A camera told to look at where it stands keeps whatever it was pointing at.
    if (direction.lengthSq() > 0) {
      this.yaw = Math.atan2(-direction.x, -direction.z);
      this.pitch = Math.atan2(direction.y, Math.hypot(direction.x, direction.z));
    }

    this.camera.quaternion.setFromEuler(this.euler.set(this.pitch, this.yaw, 0));
  }

  private axis(positive: EFlyKey, negative: EFlyKey): number {
    return Number(this.held.has(positive)) - Number(this.held.has(negative));
  }

  private readonly listeners: Readonly<Record<string, (event: IRenderProxyEvent) => void>> = {
    [ERenderInput.POINTER_DOWN]: (event: IRenderProxyEvent): void => {
      // The main button of the first pointer down alone: a second finger or another button would fight it.
      if (event.isPrimary && event.button === 0) {
        this.dragged = { id: event.pointerId, x: event.clientX, y: event.clientY };
        this.cursor.start();
      }
    },
    [ERenderInput.POINTER_MOVE]: (event: IRenderProxyEvent): void => {
      if (this.dragged?.id === event.pointerId) {
        this.lookX += event.clientX - this.dragged.x;
        this.lookY += event.clientY - this.dragged.y;
        this.dragged = { id: event.pointerId, x: event.clientX, y: event.clientY };
      }
    },
    [ERenderInput.POINTER_UP]: (event: IRenderProxyEvent): void => this.releasePointer(event),
    [ERenderInput.POINTER_CANCEL]: (event: IRenderProxyEvent): void => this.releasePointer(event),
    [ERenderInput.KEY_DOWN]: (event: IRenderProxyEvent): void => this.hold(event.code, true),
    [ERenderInput.KEY_UP]: (event: IRenderProxyEvent): void => this.hold(event.code, false),
    // A canvas that loses focus holds no key, which would otherwise fly the camera away unattended.
    [ERenderInput.BLUR]: (): void => {
      this.held.clear();
      this.release();
    },
  };

  private hold(code: string, isHeld: boolean): void {
    const key: Maybe<EFlyKey> = getFlyKey(code);

    if (key && isHeld) {
      this.held.add(key);
    } else if (key) {
      this.held.delete(key);
    }
  }

  private releasePointer(event: IRenderProxyEvent): void {
    if (this.dragged?.id === event.pointerId) {
      this.release();
    }
  }

  private release(): void {
    this.dragged = null;
    this.cursor.end();
  }
}
