import { Nullable } from "@/lib/types/general";

/** What the JavaScript heap of a Chromium-based webview currently holds. */
export interface IWebviewHeap {
  used: number;
  total: number;
  /** Ceiling the engine will not grow the heap past, which is what an out-of-memory reload would hit. */
  limit: number;
}

/** What this window can say about itself without asking the backend. */
export interface IWebviewStats {
  /**
   * Heap figures, or `null` on an engine that does not publish them.
   *
   * `performance.memory` is Chromium's and is not standard, so it answers on Windows, where the webview is WebView2,
   * and on nothing else this ships to. Reported as absent rather than as zero, because zero would read as an empty
   * heap.
   */
  heap: Nullable<IWebviewHeap>;
  /** Elements currently in the document, which is the figure a leaking surface moves. */
  nodes: number;
  /** How long the document took to load, or `null` while it still is. */
  loadDuration: Nullable<number>;
  /** Milliseconds since this document began, which a reload resets and the process behind it does not. */
  age: number;
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
 * Reads what the webview currently reports about itself.
 *
 * Every reading is guarded rather than assumed: this runs under WebView2 in the application, under WebKit on the other
 * platforms, and under jsdom in tests, and only the first publishes a heap.
 *
 * @returns What this window can say about itself.
 */
export function readWebviewStats(): IWebviewStats {
  const memory = (performance as IChromiumPerformance).memory;
  const navigation: Nullable<PerformanceNavigationTiming> = readNavigationTiming();

  return {
    heap: memory ? { used: memory.usedJSHeapSize, total: memory.totalJSHeapSize, limit: memory.jsHeapSizeLimit } : null,
    nodes: document.getElementsByTagName("*").length,
    // Zero until the load event fires, so it is reported as still loading rather than as an instant one.
    loadDuration: navigation?.loadEventEnd ? navigation.loadEventEnd - navigation.startTime : null,
    age: performance.now(),
  };
}

/**
 * The navigation entry for this document, where the engine keeps one.
 *
 * Called through rather than indexed directly because `getEntriesByType` is optional in practice: the timing buffer is
 * a separate specification from `performance` itself, and the jsdom this is tested under implements neither.
 */
function readNavigationTiming(): Nullable<PerformanceNavigationTiming> {
  const [entry] = (performance.getEntriesByType?.("navigation") ?? []) as Array<PerformanceNavigationTiming>;

  return entry ?? null;
}
