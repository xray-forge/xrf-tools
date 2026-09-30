import { Nullable } from "@xrf/types";

/**
 * The share of a run one phase took.
 *
 * @param duration - Milliseconds the phase held.
 * @param total - Milliseconds across every sampled phase.
 * @returns The share as a percentage, or null when nothing was sampled to compare it against.
 */
export function toPhaseShare(duration: number, total: number): Nullable<number> {
  return total > 0 ? (duration / total) * 100 : null;
}
