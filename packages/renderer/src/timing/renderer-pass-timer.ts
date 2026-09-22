import { toMean } from "@xrf/math";

import { IRendererPassCost } from "#/contract/renderer-report";

/** Frames averaged over, the window the frame timer uses. */
const WINDOW: number = 30;

/**
 * A rolling mean of what each pass cost on the GPU.
 */
export class RendererPassTimer {
  private readonly samples: Map<string, Array<number>> = new Map();

  /**
   * Records one frame's per-pass totals.
   *
   * @param frame - GPU milliseconds by pass name.
   */
  public record(frame: ReadonlyMap<string, number>): void {
    for (const [pass, duration] of frame) {
      const samples: Array<number> = this.samples.get(pass) ?? [];

      samples.push(duration);

      if (samples.length > WINDOW) {
        samples.shift();
      }

      this.samples.set(pass, samples);
    }
  }

  /**
   * @param passes - The frame's passes, in the order they run.
   * @returns Each pass's mean over the window, zero for one never timed.
   */
  public describe(passes: ReadonlyArray<string>): Array<IRendererPassCost> {
    return passes.map((name: string) => {
      const samples: ReadonlyArray<number> = this.samples.get(name) ?? [];

      return {
        gpuTime: toMean(samples),
        name,
      };
    });
  }

  /** Forgets the window, for a renderer that restarted. */
  public reset(): void {
    this.samples.clear();
  }
}
