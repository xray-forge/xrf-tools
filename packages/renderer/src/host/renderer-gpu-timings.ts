import { TimestampQuery } from "three/webgpu";

import { IRendererPassCost } from "#/contract/renderer-report";
import { getRendererBackend, IRendererBackend } from "#/device/renderer-backend";
import { RendererDevice } from "#/device/renderer-device";
import { RendererPassTimer } from "#/timing/renderer-pass-timer";
import { toFramePassTimes } from "#/timing/renderer-pass-times";

/**
 * What each pass cost on the GPU, read from what three resolved: one read in flight at a time, never waited on.
 */
export class RendererGpuTimings {
  private readonly timer: RendererPassTimer = new RendererPassTimer();
  /** Bumped by every reset, so a read finishing late can tell it was superseded. */
  private generation: number = 0;
  private isResolving: boolean = false;

  /**
   * @param device - The device whose renders are timed.
   */
  public resolve(device: RendererDevice): void {
    if (!device.isGpuTimed || this.isResolving) {
      return;
    }

    const generation: number = this.generation;
    const backend: IRendererBackend = getRendererBackend(device.renderer);
    const { inspector } = device;

    this.isResolving = true;

    device.renderer
      .resolveTimestampsAsync(TimestampQuery.RENDER)
      .then(() => {
        if (generation !== this.generation) {
          return;
        }

        const { frames, consumed } = toFramePassTimes(inspector.issued, (uid: string) =>
          backend.hasTimestampQuery?.(uid) ? backend.getTimestamp?.(uid) : undefined
        );

        inspector.consume(consumed);
        frames.forEach((frame: Map<string, number>) => this.timer.record(frame));
      })
      .catch(() => {})
      .finally(() => {
        if (generation === this.generation) {
          this.isResolving = false;
        }
      });
  }

  /**
   * @param passes - The frame's passes, in the order they run.
   * @returns Each pass's mean GPU time.
   */
  public describe(passes: ReadonlyArray<string>): Array<IRendererPassCost> {
    return this.timer.describe(passes);
  }

  /** Forgets every timing, for a device that went away. */
  public reset(): void {
    this.generation += 1;
    this.isResolving = false;
    this.timer.reset();
  }
}
