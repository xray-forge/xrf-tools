import { Nullable, Optional } from "@xrf/types";

/** Matches the frame a render context's timestamp uid belongs to: three spells them `<context>:f<frame>`. */
const FRAME_PATTERN: RegExp = /:f(\d+)$/;

/**
 * Sums resolved GPU durations into per-frame, per-pass totals.
 *
 * A pass may issue several renders, each timed under its own uid; the pass costs their sum, per frame.
 *
 * @param issued - Every render still waiting for its timing, by uid, with the pass that issued it.
 * @param resolve - The resolved duration of a uid in milliseconds, or undefined while it has none.
 * @returns Per-pass totals of every frame that resolved, oldest frame first, and the uids consumed.
 */
export function toFramePassTimes(
  issued: ReadonlyMap<string, string>,
  resolve: (uid: string) => Optional<number>
): { frames: Array<Map<string, number>>; consumed: Array<string> } {
  const byFrame: Map<number, Map<string, number>> = new Map();
  const consumed: Array<string> = [];

  for (const [uid, pass] of issued) {
    const duration: Optional<number> = resolve(uid);
    const match: Nullable<RegExpMatchArray> = uid.match(FRAME_PATTERN);

    if (duration === undefined || !match) {
      continue;
    }

    const frame: number = Number(match[1]);
    const passes: Map<string, number> = byFrame.get(frame) ?? new Map();

    passes.set(pass, (passes.get(pass) ?? 0) + duration);
    byFrame.set(frame, passes);
    consumed.push(uid);
  }

  return {
    consumed,
    frames: [...byFrame.entries()].sort(([left], [right]) => left - right).map(([, passes]) => passes),
  };
}
