import { formatNumber, formatSeconds } from "@xrf/format";

import { ArchiveLightAnimItem } from "@/core/ipc/types/xrf-app-archives";

/**
 * How long one colour animation runs and how fast.
 *
 * @param item - Animation to describe.
 * @returns Its length with the rate behind it, or a phrase for one that never advances.
 */
export function describeTiming(item: ArchiveLightAnimItem): string {
  const rate: string = `${formatNumber(item.fps, 0)} fps`;

  return item.durationSeconds === null
    ? `${item.frames} frames · never advances`
    : `${formatSeconds(item.durationSeconds)} · ${item.frames} frames at ${rate}`;
}
