import { ArchiveLightAnimItem } from "@/core/ipc/types/xrf-app";
import { formatSeconds } from "@/lib/format/duration";
import { formatNumber } from "@/lib/format/number";

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
