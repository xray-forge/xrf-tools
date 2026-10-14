import { describe, expect, it } from "@jest/globals";

import {
  BYTES_PER_KILOBYTE,
  BYTES_PER_MEGABYTE,
  bytesToMegabytes,
  bytesToWholeMegabytes,
  megabytesToBytes,
} from "#/byte-size";

describe("byte sizes", () => {
  it("counts binary units, not decimal ones", () => {
    expect(BYTES_PER_KILOBYTE).toBe(1024);
    expect(BYTES_PER_MEGABYTE).toBe(1024 * 1024);
  });

  it("converts without rounding, leaving that to the caller", () => {
    expect(bytesToMegabytes(BYTES_PER_MEGABYTE)).toBe(1);
    expect(bytesToMegabytes(BYTES_PER_MEGABYTE * 1.5)).toBe(1.5);
    expect(bytesToMegabytes(0)).toBe(0);
  });

  it("rounds for the fields that hold no fraction", () => {
    expect(bytesToWholeMegabytes(BYTES_PER_MEGABYTE * 1.5)).toBe(2);
    expect(bytesToWholeMegabytes(BYTES_PER_MEGABYTE * 1.4)).toBe(1);
    expect(bytesToWholeMegabytes(BYTES_PER_KILOBYTE)).toBe(0);
  });

  it("converts back to bytes", () => {
    expect(megabytesToBytes(1)).toBe(BYTES_PER_MEGABYTE);
    expect(megabytesToBytes(0)).toBe(0);
    // Round trips, which is what the volume ceiling relies on.
    expect(bytesToMegabytes(megabytesToBytes(1900))).toBe(1900);
  });
});
