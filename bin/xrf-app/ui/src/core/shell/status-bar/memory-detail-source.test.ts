import { afterEach, describe, expect, it } from "@jest/globals";

import { readPageScriptHeapDetail } from "@/core/shell/status-bar/memory-detail-source";

const MB: number = 1024 * 1024;

// Chromium's heap figures, which jsdom's `performance` lacks.
function publishHeap(used: number, total: number, limit: number): void {
  Object.defineProperty(performance, "memory", {
    configurable: true,
    value: { usedJSHeapSize: used, totalJSHeapSize: total, jsHeapSizeLimit: limit },
  });
}

describe("readPageScriptHeapDetail", () => {
  afterEach(() => {
    Reflect.deleteProperty(performance, "memory");
  });

  it("leaves the row out on an engine that publishes no heap", () => {
    expect(readPageScriptHeapDetail()).toBeNull();
  });

  it("describes the heap against its limit", () => {
    publishHeap(42 * MB, 64 * MB, 4200 * MB);

    expect(readPageScriptHeapDetail()).toEqual({ label: "Script heap", value: "42 MB / 4.1 GB" });
  });
});
