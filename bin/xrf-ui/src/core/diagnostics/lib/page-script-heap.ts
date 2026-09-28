import { Nullable } from "@xrf/types";

import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { formatBytes } from "@/lib/memory/format";

/** What the page's JavaScript heap holds, as a Chromium-based webview reports it. */
export interface IPageScriptHeap {
  used: number;
  total: number;
  /** Ceiling the engine will not grow the heap past, which is what an out-of-memory reload would hit. */
  limit: number;
}

/** The non-standard shape Chromium hangs off `performance`, named here because no lib declaration carries it. */
interface IChromiumPerformance extends Performance {
  memory?: {
    usedJSHeapSize: number;
    totalJSHeapSize: number;
    jsHeapSizeLimit: number;
  };
}

/**
 * Reads the page's script heap.
 *
 * `performance.memory` is Chromium's and not standard: it answers under WebView2 and not under WebKit or jsdom, and is
 * reported absent there rather than as zero, which would read as an empty heap.
 *
 * @returns The heap, or `null` on an engine that publishes none.
 */
export function readPageScriptHeap(): Nullable<IPageScriptHeap> {
  const memory: IChromiumPerformance["memory"] = (performance as IChromiumPerformance).memory;

  return memory ? { used: memory.usedJSHeapSize, total: memory.totalJSHeapSize, limit: memory.jsHeapSizeLimit } : null;
}

/**
 * Memory hover row for the page's script heap against its limit.
 *
 * @returns The row, or `null` on an engine that publishes no heap.
 */
export function readPageScriptHeapDetail(): Nullable<IEditorStatusDetail> {
  const heap: Nullable<IPageScriptHeap> = readPageScriptHeap();

  return heap ? { label: "Script heap", value: `${formatBytes(heap.used)} / ${formatBytes(heap.limit)}` } : null;
}
