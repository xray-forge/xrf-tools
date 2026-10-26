import { describe, expect, it } from "@jest/globals";

import { readWebviewStats } from "./webview-stats";

describe("readWebviewStats", () => {
  it("reports an engine that publishes no heap as absent rather than as an empty one", () => {
    // jsdom stands in for WebKit here: neither hangs `memory` off `performance`, which is what the guard is for.
    expect(readWebviewStats().heap).toBeNull();
  });

  it("reports a document with no navigation timing as still loading rather than as instant", () => {
    expect(readWebviewStats().loadDuration).toBeNull();
  });

  it("counts what the document currently holds", () => {
    expect(readWebviewStats().nodes).toBe(document.getElementsByTagName("*").length);
  });
});
