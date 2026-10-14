import { describe, expect, it } from "@jest/globals";

import { formatBytes, formatBytesPair } from "#/bytes";

describe("formatBytes", () => {
  it("uses adaptive binary units and compact precision", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(1536)).toBe("1.5 KB");
    expect(formatBytes(10 * 1024 * 1024)).toBe("10 MB");
    expect(formatBytes(1536 * 1024 * 1024)).toBe("1.5 GB");
  });
});

describe("formatBytesPair", () => {
  it("formats both counts in the larger value's unit", () => {
    expect(formatBytesPair(1198 * 1024 * 1024, 800 * 1024 * 1024)).toEqual(["1.17 GB", "0.78 GB"]);
    expect(formatBytesPair(512, 512)).toEqual(["512 B", "512 B"]);
  });

  it("treats negative counts as zero", () => {
    expect(formatBytesPair(-1, 1536)).toEqual(["0 KB", "1.5 KB"]);
  });
});
