import { afterEach, describe, expect, it } from "@jest/globals";

import { readPageScriptHeap, readPageScriptHeapDetail } from "@/core/diagnostics/lib/page-script-heap";

const MB: number = 1024 * 1024;

/** Hangs Chromium's heap figures off `performance`, which jsdom leaves bare. */
function publishHeap(used: number, total: number, limit: number): void {
  Object.defineProperty(performance, "memory", {
    configurable: true,
    value: { usedJSHeapSize: used, totalJSHeapSize: total, jsHeapSizeLimit: limit },
  });
}

describe("readPageScriptHeap", () => {
  afterEach(() => {
    Reflect.deleteProperty(performance, "memory");
  });

  it("reports an engine that publishes no heap as absent rather than as an empty one", () => {
    expect(readPageScriptHeap()).toBeNull();
    expect(readPageScriptHeapDetail()).toBeNull();
  });

  it("reads the heap Chromium publishes", () => {
    publishHeap(42 * MB, 64 * MB, 4200 * MB);

    expect(readPageScriptHeap()).toEqual({ used: 42 * MB, total: 64 * MB, limit: 4200 * MB });
  });

  it("describes the heap against its limit", () => {
    publishHeap(42 * MB, 64 * MB, 4200 * MB);

    expect(readPageScriptHeapDetail()).toEqual({ label: "Script heap", value: "42 MB / 4.1 GB" });
  });
});
