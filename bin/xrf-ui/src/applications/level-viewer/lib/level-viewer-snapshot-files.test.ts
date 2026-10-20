import { describe, expect, it } from "@jest/globals";

import { toLevelViewerSnapshotName } from "@/applications/level-viewer/lib/level-viewer-snapshot-files";

describe("level view snapshot files", () => {
  it("names a file for its level and moment, sorting by time within a level", () => {
    const at: Date = new Date(2026, 9, 20, 7, 5, 9);

    expect(toLevelViewerSnapshotName("levels/zaton", at)).toBe("zaton-20261020-070509.json");
    expect(toLevelViewerSnapshotName(null, at)).toBe("view-20261020-070509.json");
  });
});
