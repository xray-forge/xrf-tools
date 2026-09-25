import { toMean } from "@xrf/math";

import { IRendererPassCost } from "#/contract/renderer-report";

/** Frames averaged over, the window the frame timer uses. */
const WINDOW: number = 30;

/**
 * A rolling mean of what each pass cost on the GPU a frame. A frame a pass issued nothing in costs it nothing, so a pass
 * drawing only now and then, a staggered cascade or a shadow face drawn once and kept, shows what it costs a frame
 * rather than what it cost the last time it drew.
 */
export class RendererPassTimer {
  private readonly samples: Map<string, Array<number>> = new Map();

  /**
   * Records one frame's per-pass totals.
   *
   * @param frame - GPU milliseconds by pass name.
   */
  public record(frame: ReadonlyMap<string, number>): void {
    for (const pass of frame.keys()) {
      if (!this.samples.has(pass)) {
        this.samples.set(pass, []);
      }
    }

    for (const [pass, samples] of this.samples) {
      samples.push(frame.get(pass) ?? 0);

      if (samples.length > WINDOW) {
        samples.shift();
      }
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
