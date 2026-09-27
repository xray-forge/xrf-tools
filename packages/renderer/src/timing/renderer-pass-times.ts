import { Optional } from "@xrf/types";

import { TIssuedRender } from "#/timing/renderer-pass-inspector";

/**
 * Sums resolved GPU durations into per-frame, per-pass totals.
 *
 * A pass may issue several renders, each timed under its own uid; the pass costs their sum, per frame. Three resolves
 * a frame's renders together, so a frame that resolved at all is read whole and let go of: a render in it without a
 * duration is one three never timed.
 *
 * @param issued - Every render still waiting for its timing, by the frame it was issued in, oldest first.
 * @param resolve - The resolved duration of a uid in milliseconds, or undefined while it has none.
 * @returns Per-pass totals of every frame that resolved, oldest frame first, and those frames.
 */
export function toFramePassTimes(
  issued: ReadonlyMap<number, ReadonlyArray<TIssuedRender>>,
  resolve: (uid: string) => Optional<number>
): { frames: Array<Map<string, number>>; consumed: Array<number> } {
  const frames: Array<Map<string, number>> = [];
  const consumed: Array<number> = [];

  for (const [frame, renders] of issued) {
    const passes: Map<string, number> = new Map();

    for (const [uid, pass] of renders) {
      const duration: Optional<number> = resolve(uid);

      if (duration !== undefined) {
        passes.set(pass, (passes.get(pass) ?? 0) + duration);
      }
    }

    if (passes.size) {
      frames.push(passes);
      consumed.push(frame);
    }
  }

  return { consumed, frames };
}
