import { Camera, Euler, Vector3 } from "three";

import { ILevelFlyInput } from "./level-fly-input";
import { DEFAULT_LEVEL_FLY_OPTIONS, ILevelFlyOptions } from "./level-fly-options";

/** Just short of straight up, so looking at the sky never flips the horizon over. */
const MAX_PITCH: number = Math.PI / 2 - 0.001;

/**
 * Free camera for walking a level: yaw and pitch from the pointer, translation from the keys.
 */
export class LevelFlyCamera {
  private yaw: number = 0;
  private pitch: number = 0;

  public options: ILevelFlyOptions = DEFAULT_LEVEL_FLY_OPTIONS;

  /**
   * Turns the camera by a pointer movement.
   *
   * @param deltaX - Pointer movement across, in pixels.
   * @param deltaY - Pointer movement down, in pixels.
   */
  public look(deltaX: number, deltaY: number): void {
    this.yaw -= deltaX * this.options.sensitivity;
    this.pitch = Math.max(-MAX_PITCH, Math.min(MAX_PITCH, this.pitch - deltaY * this.options.sensitivity));
  }

  /**
   * Moves the camera for one frame and writes the result onto it.
   *
   * @param camera - Camera to drive.
   * @param input - Keys currently held.
   * @param delta - Seconds since the previous frame.
   * @returns Whether the camera moved or turned, so a caller can skip work when it did not.
   */
  public update(camera: Camera, input: ILevelFlyInput, delta: number): boolean {
    const forward: number = Number(input.forward) - Number(input.back);
    const strafe: number = Number(input.right) - Number(input.left);
    const rise: number = Number(input.up) - Number(input.down);

    camera.quaternion.setFromEuler(new Euler(this.pitch, this.yaw, 0, "YXZ"));

    if (!forward && !strafe && !rise) {
      return false;
    }

    const distance: number = this.options.speed * (input.fast ? this.options.boost : 1) * delta;
    const ahead: Vector3 = new Vector3(0, 0, -1).applyQuaternion(camera.quaternion);
    const across: Vector3 = new Vector3(1, 0, 0).applyQuaternion(camera.quaternion);

    camera.position
      .addScaledVector(ahead, forward * distance)
      .addScaledVector(across, strafe * distance)
      // World up rather than the camera's, so rising is always rising even when looking down.
      .addScaledVector(new Vector3(0, 1, 0), rise * distance);

    return true;
  }

  /**
   * Points the camera at a position from where it already is.
   *
   * @param camera - Camera to aim.
   * @param target - What to look at.
   */
  public lookAt(camera: Camera, target: Vector3): void {
    const direction: Vector3 = new Vector3().subVectors(target, camera.position);

    if (direction.lengthSq() === 0) {
      return;
    }

    this.yaw = Math.atan2(-direction.x, -direction.z);
    this.pitch = Math.atan2(direction.y, Math.hypot(direction.x, direction.z));

    camera.quaternion.setFromEuler(new Euler(this.pitch, this.yaw, 0, "YXZ"));
  }
}
