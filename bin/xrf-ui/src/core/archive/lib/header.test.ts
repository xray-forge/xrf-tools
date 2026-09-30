import { describe, expect, it } from "@jest/globals";

import {
  HEADER_AUTO_LOAD,
  HEADER_ENTRY_POINT,
  isReservedHeaderKey,
  readHeaderFlag,
  readHeaderValue,
  writeHeaderValue,
} from "@/core/archive/lib/header";

const HEADER: string = "[header]\r\nAuto_Load = TRUE\r\nEntry_Point = $fs_root$\\gamedata\\\r\nlevel_name = zaton\r\n";

describe("readHeaderValue", () => {
  // The engine lower-cases the section before reading it, so a key written in any case is the one it reads.
  it("reads a key written in any case", () => {
    expect(readHeaderValue(HEADER, HEADER_ENTRY_POINT)).toBe("$fs_root$\\gamedata\\");
    expect(readHeaderFlag(HEADER, HEADER_AUTO_LOAD)).toBe(true);
  });

  it("answers null for a key the header does not hold", () => {
    expect(readHeaderValue(HEADER, "patch")).toBeNull();
    expect(readHeaderValue(null, HEADER_ENTRY_POINT)).toBeNull();
  });
});

describe("writeHeaderValue", () => {
  it("replaces a key written in another case rather than adding a second one", () => {
    const written: string = writeHeaderValue(HEADER, HEADER_ENTRY_POINT, "$game_data$") ?? "";

    expect(written.match(/entry_point/gi)).toHaveLength(1);
    expect(readHeaderValue(written, HEADER_ENTRY_POINT)).toBe("$game_data$");
  });

  it("removes a key for a blank value, and the whole header when nothing is left", () => {
    expect(writeHeaderValue("[header]\r\nEntry_Point = x\r\n", HEADER_ENTRY_POINT, " ")).toBeNull();
  });
});

describe("isReservedHeaderKey", () => {
  it("knows the keys an editor gives a control of their own, in any case", () => {
    expect(isReservedHeaderKey("ENTRY_POINT")).toBe(true);
    expect(isReservedHeaderKey("level_name")).toBe(false);
  });
});
