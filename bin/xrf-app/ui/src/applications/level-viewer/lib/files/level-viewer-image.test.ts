import { describe, expect, it } from "@jest/globals";

import { readPngSize } from "@/applications/level-viewer/lib/files/level-viewer-image";

describe("level viewer image", () => {
  it("reads a PNG's size from its header", () => {
    const header: Uint8Array = Uint8Array.from([
      137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 15, 0, 0, 0, 8, 112,
    ]);

    expect(readPngSize(header)).toEqual([3840, 2160]);
  });
});
