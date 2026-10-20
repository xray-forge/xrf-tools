import { describe, expect, it } from "@jest/globals";

import {
  readPngSize,
  toLevelViewerScreenshotReportPath,
} from "@/applications/level-viewer/lib/screenshot/level-viewer-screenshot";

describe("level viewer screenshot", () => {
  it("reads a PNG's size from its header and names its report beside it", () => {
    const header: Uint8Array = Uint8Array.from([
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 15, 0, 0, 0, 8, 112,
    ]);

    expect(readPngSize(header)).toEqual([3840, 2160]);
    expect(toLevelViewerScreenshotReportPath("C:/appdata/zaton-1.png")).toBe("C:/appdata/zaton-1.json");
  });
});
