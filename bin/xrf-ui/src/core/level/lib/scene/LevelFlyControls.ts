import { EMPTY_LEVEL_FLY_INPUT, getFlyBinding, ILevelFlyInput } from "@/core/level/lib/camera";
import { ILevelFlyMotion, ILevelMotionSource } from "@/core/level/lib/camera/level-fly-motion";
import { DRAG_CURSOR } from "@/lib/media/drag-cursor";

/**
 * Binds a viewport's pointer and keyboard to whatever reads them.
 */
export class LevelFlyControls implements ILevelMotionSource {
  private readonly element: HTMLElement;
  private readonly input: ILevelFlyInput = { ...EMPTY_LEVEL_FLY_INPUT };

  /** Pointer movement gathered since the last drain, because a frame is what applies it. */
  private lookX: number = 0;
  private lookY: number = 0;

  private isLooking: boolean = false;
  /** The cursor the element had before a drag took it, so letting go puts back whatever was there. */
  private restingCursor: string = "";

  /**
   * @param element - Element the viewport draws into, which is what takes focus and receives the events.
   */
  public constructor(element: HTMLElement) {
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
   * @returns What has happened since the last call, the look forgotten as it is handed over.
   */
  public drain(): ILevelFlyMotion {
    const motion: ILevelFlyMotion = { keys: { ...this.input }, lookX: this.lookX, lookY: this.lookY };

    this.lookX = 0;
    this.lookY = 0;

    return motion;
  }

  /** Stops listening, for a viewport being torn down. */
  public dispose(): void {
    this.element.removeEventListener("pointerdown", this.onPointerDown);
    this.element.removeEventListener("pointermove", this.onPointerMove);
    this.element.removeEventListener("keydown", this.onKeyDown);
    this.element.removeEventListener("keyup", this.onKeyUp);
    this.element.removeEventListener("blur", this.onBlur);

    window.removeEventListener("pointerup", this.onPointerUp);

    this.element.style.cursor = this.restingCursor;
  }

  private readonly onPointerDown = (event: PointerEvent): void => {
    this.isLooking = true;
    this.element.setPointerCapture(event.pointerId);
    this.element.focus();

    // The same answer the orbit previews give the same gesture: a drag that turns the view says so on the pointer.
    this.restingCursor = this.element.style.cursor;
    this.element.style.cursor = DRAG_CURSOR;
  };

  private readonly onPointerUp = (): void => {
    this.isLooking = false;
    this.element.style.cursor = this.restingCursor;
  };

  private readonly onPointerMove = (event: PointerEvent): void => {
    if (this.isLooking) {
      this.lookX += event.movementX;
      this.lookY += event.movementY;
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
    this.lookX = 0;
    this.lookY = 0;

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
