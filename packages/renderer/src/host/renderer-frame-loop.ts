import { Nullable } from "@xrf/types";

/** Schedules a callback for the next frame, as `requestAnimationFrame` does. */
export type TRendererFrameScheduler = (callback: (now: number) => void) => number;

/**
 * The frame loop: at most one frame scheduled at a time, each told the time it runs at.
 */
export class RendererFrameLoop {
  private readonly schedule: TRendererFrameScheduler;
  private readonly cancelFrame: (handle: number) => void;
  private readonly onFrame: (now: number) => void;
  private handle: Nullable<number> = null;

  public constructor(
    schedule: TRendererFrameScheduler,
    cancel: (handle: number) => void,
    onFrame: (now: number) => void
  ) {
    this.schedule = schedule;
    this.cancelFrame = cancel;
    this.onFrame = onFrame;
  }

  /** Schedules the next frame, unless one already is. */
  public request(): void {
    this.handle ??= this.schedule(this.run);
  }

  public cancel(): void {
    if (this.handle !== null) {
      this.cancelFrame(this.handle);
      this.handle = null;
    }
  }

  private readonly run = (now: number): void => {
    this.handle = null;
    this.onFrame(now);
  };
}
