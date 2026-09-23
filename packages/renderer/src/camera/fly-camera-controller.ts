import { Maybe, Nullable } from "@xrf/types";
import { Euler, PerspectiveCamera, Vector3 } from "three/webgpu";

import { IRendererCameraController } from "#/camera/camera-controller";
import { DRAG_CURSOR } from "#/camera/drag-cursor";
import { EFlyKey, getFlyKey } from "#/camera/fly-keys";
import {
  ERendererCameraCommand,
  ERendererCameraController,
  IRendererCameraPose,
  IRendererFlyCamera,
  TRendererCamera,
  TRendererCameraCommand,
} from "#/contract/renderer-camera";
import { TRendererVector } from "#/contract/renderer-lighting";
import { ERenderInput } from "#/input/render-input";
import { IRenderProxyEvent, RenderProxyElement } from "#/input/render-proxy-element";

/** Just short of straight up, so looking at the sky never flips the horizon over. */
const MAX_PITCH: number = Math.PI / 2 - 0.001;

/** The longest step one frame moves by, so a frame after a stall does not throw the camera across the level. */
const MAX_DELTA: number = 0.1;

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

const UP: Vector3 = new Vector3(0, 1, 0);

/**
 * A free camera: a drag turns it, the keys move it along where it faces, and rising is always along world up.
 * It holds yaw and pitch itself, since a rotation read back off the camera cannot tell `+π` from `-π`.
 */
export class FlyCameraController implements IRendererCameraController {
  public readonly camera: PerspectiveCamera = new PerspectiveCamera();

  private readonly element: RenderProxyElement;
  private readonly held: Set<EFlyKey> = new Set();
  private readonly euler: Euler = new Euler(0, 0, 0, "YXZ");
  private readonly ahead: Vector3 = new Vector3();
  private readonly across: Vector3 = new Vector3();

  private description: IRendererFlyCamera = DEFAULT_FLY_CAMERA;
  private yaw: number = 0;
  private pitch: number = 0;
  /** Pointer movement gathered since the last frame, which is what applies it. */
  private lookX: number = 0;
  private lookY: number = 0;
  /** Where the dragging pointer last was, or null while nothing drags. */
  private dragged: Nullable<{ x: number; y: number }> = null;
  /** The cursor the element had before a drag took it. */
  private restingCursor: string = "";

  public constructor(element: RenderProxyElement) {
    this.element = element;

    for (const [type, listener] of Object.entries(this.listeners)) {
      element.addEventListener(type, listener);
    }

    this.describe(DEFAULT_FLY_CAMERA);
  }

  public describe(description: TRendererCamera): void {
    if (description.kind !== ERendererCameraController.FLY) {
      return;
    }

    const isMoved: boolean =
      !isSameVector(description.position, this.description.position) ||
      !isSameVector(description.target, this.description.target);

    this.description = description;
    this.camera.fov = description.fieldOfView;
    this.camera.near = description.near;
    this.camera.far = description.far;
    this.camera.updateProjectionMatrix();

    // The same start again is new speeds or a new lens, not a request to go back.
    if (isMoved || description === DEFAULT_FLY_CAMERA) {
      this.reset();
    }
  }

  public command(command: TRendererCameraCommand): void {
    if (command.kind === ERendererCameraCommand.RESET) {
      this.reset();
    }
  }

  public resize(width: number, height: number): void {
    const aspect: number = width / Math.max(height, 1);

    if (aspect !== this.camera.aspect) {
      this.camera.aspect = aspect;
      this.camera.updateProjectionMatrix();
    }
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
    this.camera.position
      .addScaledVector(this.ahead, forward * distance)
      .addScaledVector(this.across, strafe * distance)
      .addScaledVector(UP, rise * distance);
  }

  public get pose(): IRendererCameraPose {
    const { position } = this.camera;
    const ahead: Vector3 = new Vector3(0, 0, -1).applyQuaternion(this.camera.quaternion).add(position);

    return { position: [position.x, position.y, position.z], target: [ahead.x, ahead.y, ahead.z] };
  }

  public dispose(): void {
    for (const [type, listener] of Object.entries(this.listeners)) {
      this.element.removeEventListener(type, listener);
    }

    if (this.dragged) {
      this.element.style.cursor = this.restingCursor;
    }
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
      this.dragged = { x: event.clientX, y: event.clientY };
      this.restingCursor = this.element.style.cursor;
      this.element.style.cursor = DRAG_CURSOR;
    },
    [ERenderInput.POINTER_MOVE]: (event: IRenderProxyEvent): void => {
      if (this.dragged) {
        this.lookX += event.clientX - this.dragged.x;
        this.lookY += event.clientY - this.dragged.y;
        this.dragged = { x: event.clientX, y: event.clientY };
      }
    },
    [ERenderInput.POINTER_UP]: (): void => this.release(),
    [ERenderInput.POINTER_CANCEL]: (): void => this.release(),
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

  private release(): void {
    if (this.dragged) {
      this.dragged = null;
      this.element.style.cursor = this.restingCursor;
    }
  }
}

function isSameVector(left: TRendererVector, right: TRendererVector): boolean {
  return left[0] === right[0] && left[1] === right[1] && left[2] === right[2];
}
