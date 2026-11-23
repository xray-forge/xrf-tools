import { describe, expect, it } from "@jest/globals";
import { ArchivePackHeader } from "@xrf/ipc/types/xrf-pack";

import {
  HEADER_AUTO_LOAD,
  HEADER_ENTRY_POINT,
  isReservedHeaderKey,
  readHeaderFlag,
  readHeaderValue,
  removeHeaderEntry,
  writeHeaderValue,
} from "@/core/archive/lib/header";

const HEADER: ArchivePackHeader = [
  { key: "Auto_Load", value: "TRUE" },
  { key: "Entry_Point", value: "$fs_root$\\gamedata\\" },
  { key: "level_name", value: "zaton" },
];

describe("readHeaderValue", () => {
  it("reads a key written in any case", () => {
    expect(readHeaderValue(HEADER, HEADER_ENTRY_POINT)).toBe("$fs_root$\\gamedata\\");
    expect(readHeaderFlag(HEADER, HEADER_AUTO_LOAD)).toBe(true);
  });

  it("answers null for a key the header does not hold, or holds blank", () => {
    expect(readHeaderValue(HEADER, "patch")).toBeNull();
    expect(readHeaderValue(null, HEADER_ENTRY_POINT)).toBeNull();
    expect(readHeaderValue([{ key: HEADER_ENTRY_POINT, value: "" }], HEADER_ENTRY_POINT)).toBeNull();
  });
});

describe("writeHeaderValue", () => {
  it("replaces a key written in another case with the spelling given, rather than adding a second one", () => {
    const written: ArchivePackHeader = writeHeaderValue(HEADER, HEADER_ENTRY_POINT, "$game_data$\\");

    expect(written.filter((entry) => entry.key.toLowerCase() === HEADER_ENTRY_POINT)).toEqual([
      { key: HEADER_ENTRY_POINT, value: "$game_data$\\" },
    ]);
  });

  it("replaces a key where it stands, so the values keep the order they were written in", () => {
    expect(writeHeaderValue(HEADER, HEADER_AUTO_LOAD, "false").map((entry) => entry.key)).toEqual([
      HEADER_AUTO_LOAD,
      "Entry_Point",
      "level_name",
    ]);
  });

  it("appends a key the header does not hold, starting a header where there was none", () => {
    expect(writeHeaderValue(HEADER, "patch", "1")).toEqual([...HEADER, { key: "patch", value: "1" }]);
    expect(writeHeaderValue(null, "patch", "1")).toEqual([{ key: "patch", value: "1" }]);
  });

  // Trimmed or dropped on every keystroke, a value could never hold a space nor be cleared to retype.
  it("keeps a value exactly as typed, blank included", () => {
    expect(readHeaderValue(writeHeaderValue(HEADER, "creator", '"My Mod"'), "creator")).toBe('"My Mod"');
    expect(writeHeaderValue(HEADER, "level_name", "")).toContainEqual({ key: "level_name", value: "" });
  });

  it("collapses a key repeated in any case into its first entry", () => {
    const repeated: ArchivePackHeader = [...HEADER, { key: "LEVEL_NAME", value: "jupiter" }];

    expect(writeHeaderValue(repeated, "level_name", "pripyat")).toEqual([
      HEADER[0],
      HEADER[1],
      { key: "level_name", value: "pripyat" },
    ]);
  });
});

describe("removeHeaderEntry", () => {
  it("removes a key in any case and leaves the rest in order", () => {
    expect(removeHeaderEntry(HEADER, "LEVEL_NAME")).toEqual([HEADER[0], HEADER[1]]);
  });

  it("drops the whole header when nothing is left", () => {
    expect(removeHeaderEntry([{ key: "author", value: "XRF" }], "author")).toBeNull();
  });
});

describe("isReservedHeaderKey", () => {
  it("knows the keys an editor gives a control of their own, in any case", () => {
    expect(isReservedHeaderKey("ENTRY_POINT")).toBe(true);
    expect(isReservedHeaderKey("level_name")).toBe(false);
  });
});
