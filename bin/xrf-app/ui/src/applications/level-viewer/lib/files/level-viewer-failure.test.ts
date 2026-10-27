import { describe, expect, it } from "@jest/globals";

import { describeLevelViewerFailure } from "@/applications/level-viewer/lib/files/level-viewer-failure";

describe("describeLevelViewerFailure", () => {
  it("says what failed and why, from an error or a rejection the backend answered with text", () => {
    expect(describeLevelViewerFailure("export the view", new Error("the disk is full"))).toBe(
      "Failed to export the view: the disk is full"
    );
    expect(describeLevelViewerFailure("delete the screenshot", "the file is open elsewhere")).toBe(
      "Failed to delete the screenshot: the file is open elsewhere"
    );
  });
});
