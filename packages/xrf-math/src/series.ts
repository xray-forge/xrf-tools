/**
 * The mean of a series.
 *
 * @param samples - What was measured, in any order.
 * @returns Their mean, or zero for an empty series, since a reading of nothing is not a reading of anything else.
 */
export function toMean(samples: ReadonlyArray<number>): number {
  if (!samples.length) {
    return 0;
  }

  return samples.reduce((total: number, sample: number) => total + sample, 0) / samples.length;
}

/**
 * The largest of a series of costs, reported beside a mean because a mean hides a stutter.
 *
 * @param samples - What was measured, in any order.
 * @returns The largest, or zero for an empty series.
 */
export function toWorst(samples: ReadonlyArray<number>): number {
  return samples.reduce((worst: number, sample: number) => Math.max(worst, sample), 0);
}
