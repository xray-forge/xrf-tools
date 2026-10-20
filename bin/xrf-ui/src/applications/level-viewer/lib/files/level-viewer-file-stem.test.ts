import { describe, expect, it } from "@jest/globals";

import { toLevelViewerFileStem } from "@/applications/level-viewer/lib/files/level-viewer-file-stem";

describe("level viewer file stem", () => {
  it("names a file for its level and moment, sorting by time within a level", () => {
    const at: Date = new Date(2026, 9, 20, 7, 5, 9);

    expect(toLevelViewerFileStem("levels/zaton", at)).toBe("zaton-20261020-070509");
    expect(toLevelViewerFileStem(null, at)).toBe("view-20261020-070509");
  });
});
