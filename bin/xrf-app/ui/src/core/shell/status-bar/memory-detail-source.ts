import { formatBytes } from "@xrf/format";
import { Nullable } from "@xrf/types";

import { IPageScriptHeap, readPageScriptHeap } from "@/core/diagnostics/lib/page-script-heap";
import { IShellStatusDetail } from "@/core/shell/status-bar/shell-status-detail";

/**
 * One more row for the memory hover, after the processes, read each time the processes are.
 *
 * @returns The row, or `null` where the source has nothing to say.
 */
export type TMemoryDetailSource = () => Nullable<IShellStatusDetail>;

/**
 * Memory hover row for the page's script heap against its limit.
 *
 * @returns The row, or `null` on an engine that publishes no heap.
 */
export function readPageScriptHeapDetail(): Nullable<IShellStatusDetail> {
  const heap: Nullable<IPageScriptHeap> = readPageScriptHeap();

  return heap ? { label: "Script heap", value: `${formatBytes(heap.used)} / ${formatBytes(heap.limit)}` } : null;
}
