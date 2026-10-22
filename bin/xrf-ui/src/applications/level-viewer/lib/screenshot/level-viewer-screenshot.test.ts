import { describe, expect, it } from "@jest/globals";

import { toLevelViewerScreenshotReportPath } from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";

describe("level viewer screenshot", () => {
  it("names its report beside it", () => {
    expect(toLevelViewerScreenshotReportPath("C:/appdata/zaton-1.png")).toBe("C:/appdata/zaton-1.json");
  });
});
