import { Camera } from "three";

import {
  EMPTY_LEVEL_FLY_INPUT,
  getFlyBinding,
  ILevelFlyInput,
  LevelFlyCamera,
} from "@/core/level/lib/camera/level-fly-camera";

/**
 * Binds a viewport's pointer and keyboard to a fly camera.
 */
export class LevelFlyControls {
  private readonly camera: LevelFlyCamera;
  private readonly element: HTMLElement;
  private readonly input: ILevelFlyInput = { ...EMPTY_LEVEL_FLY_INPUT };

  private isLooking: boolean = false;

  /**
   * @param camera - The fly camera to drive, which outlives these controls: where it is looking survives a viewport
   *   being unmounted and mounted again, as react's strict mode does on every render pass.
   * @param element - Element the viewport draws into, which is what takes focus and receives the events.
   */
  public constructor(camera: LevelFlyCamera, element: HTMLElement) {
    this.camera = camera;
    this.element = element;

    this.element.addEventListener("pointerdown", this.onPointerDown);
    this.element.addEventListener("pointermove", this.onPointerMove);
    this.element.addEventListener("keydown", this.onKeyDown);
    this.element.addEventListener("keyup", this.onKeyUp);
    this.element.addEventListener("blur", this.onBlur);

    // On the window rather than the element: a drag that ends outside it would otherwise leave the camera turning.
    window.addEventListener("pointerup", this.onPointerUp);
  }

  /**
   * Moves the camera for one frame.
   *
   * @param camera - Camera to drive.
   * @param delta - Seconds since the previous frame.
   * @returns Whether the camera moved.
   */
  public update(camera: Camera, delta: number): boolean {
    return this.camera.update(camera, this.input, delta);
  }

  /** Stops listening, for a viewport being torn down. */
  public dispose(): void {
    this.element.removeEventListener("pointerdown", this.onPointerDown);
    this.element.removeEventListener("pointermove", this.onPointerMove);
    this.element.removeEventListener("keydown", this.onKeyDown);
    this.element.removeEventListener("keyup", this.onKeyUp);
    this.element.removeEventListener("blur", this.onBlur);

    window.removeEventListener("pointerup", this.onPointerUp);
  }

  private readonly onPointerDown = (event: PointerEvent): void => {
    this.isLooking = true;
    this.element.setPointerCapture(event.pointerId);
    this.element.focus();
  };

  private readonly onPointerUp = (): void => {
    this.isLooking = false;
  };

  private readonly onPointerMove = (event: PointerEvent): void => {
    if (this.isLooking) {
      this.camera.look(event.movementX, event.movementY);
    }
  };

  private readonly onKeyDown = (event: KeyboardEvent): void => {
    this.setInput(event, true);
  };

  private readonly onKeyUp = (event: KeyboardEvent): void => {
    this.setInput(event, false);
  };

  /** A viewport that loses focus holds no key, which would otherwise fly the camera away unattended. */
  private readonly onBlur = (): void => {
    this.isLooking = false;

    for (const key of Object.keys(this.input) as Array<keyof ILevelFlyInput>) {
      this.input[key] = false;
    }
  };

  private setInput(event: KeyboardEvent, held: boolean): void {
    const binding: keyof ILevelFlyInput | null = getFlyBinding(event.code);

    if (binding) {
      event.preventDefault();
      this.input[binding] = held;
    }
  }
}
