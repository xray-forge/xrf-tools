import { describe, expect, it } from "@jest/globals";

import {
  findLastSeparator,
  getPathDirectory,
  getPathName,
  isSamePath,
  LOGICAL_PATH_SEPARATOR,
  splitAfterSeparators,
  toComparablePath,
  truncatePathHead,
} from "@/lib/path/separator";

describe("findLastSeparator", () => {
  it("finds either style, which is what a path from the host may be written in", () => {
    expect(findLastSeparator("C:\\gamedata\\textures\\act.dds")).toBe(20);
    expect(findLastSeparator("/home/user/textures/act.dds")).toBe(19);
  });

  it("finds the last separator of a path written in both", () => {
    // Windows hands back either, so a path can carry both: what matters is where it stops being a directory.
    expect(getPathName("C:/gamedata\\textures/act.dds")).toBe("act.dds");
  });

  it("says a path with no separator has none", () => {
    expect(findLastSeparator("act.dds")).toBe(-1);
    expect(getPathName("act.dds")).toBe("act.dds");
  });
});

describe("splitAfterSeparators", () => {
  it("keeps every character, separators included", () => {
    const path: string = "aaz\\actor\\act_aaz_svoboda4";

    expect(splitAfterSeparators(path).join("")).toBe(path);
  });

  it("cuts after each separator, so a break lands where a reader would put one", () => {
    expect(splitAfterSeparators("aaz\\actor\\act_aaz_svoboda4")).toEqual(["aaz\\", "actor\\", "act_aaz_svoboda4"]);
  });

  it("leaves a name with nothing to cut on whole", () => {
    expect(splitAfterSeparators("act_aaz_svoboda4")).toEqual(["act_aaz_svoboda4"]);
  });
});

describe("LOGICAL_PATH_SEPARATOR", () => {
  it("is the engine's own, which is never the host's forward slash", () => {
    // An engine reference is a name the game data carries rather than a path a filesystem resolved, so it is spelled
    // the same way on every platform.
    expect(LOGICAL_PATH_SEPARATOR).toBe("\\");
  });
});

describe("getPathDirectory", () => {
  it("answers the directory above a path, in either style", () => {
    expect(getPathDirectory("C:\\gamedata\\configs\\system.ltx")).toBe("C:\\gamedata\\configs");
    expect(getPathDirectory("/home/user/textures/act.dds")).toBe("/home/user/textures");
  });

  it("keeps the separator that closes a root, because a drive without it names something else", () => {
    expect(getPathDirectory("C:\\gamedata")).toBe("C:\\");
    expect(getPathDirectory("/gamedata")).toBe("/");
  });

  it("answers nothing for a bare name, which names no location", () => {
    expect(getPathDirectory("act.dds")).toBe("");
  });
});

describe("toComparablePath", () => {
  it("folds the spellings windows accepts for one directory together", () => {
    const expected: string = "c:\\projects\\gamedata";

    expect(toComparablePath("C:\\Projects\\gamedata")).toBe(expected);
    expect(toComparablePath("C:/Projects/gamedata")).toBe(expected);
    expect(toComparablePath("c:\\projects\\gamedata\\")).toBe(expected);
    expect(toComparablePath("C:\\Projects/gamedata//")).toBe(expected);
  });

  it("keeps a root a root, because a drive without its separator names something else", () => {
    expect(toComparablePath("C:\\")).toBe("c:\\");
    expect(toComparablePath("C:/")).toBe("c:\\");
    expect(toComparablePath("/")).toBe("\\");
  });

  it("tells different paths apart", () => {
    expect(toComparablePath("C:\\gamedata")).not.toBe(toComparablePath("C:\\gamedata-anomaly"));
  });

  it("leaves repeated separators inside a path alone, so a unc prefix survives", () => {
    expect(toComparablePath("\\\\server\\share\\configs")).toBe("\\\\server\\share\\configs");
  });
});

describe("isSamePath", () => {
  it("compares by the folded spelling, so one directory is one path", () => {
    expect(isSamePath("C:\\Projects\\Gamedata", "c:/projects/gamedata\\")).toBe(true);
  });

  it("tells two paths apart", () => {
    expect(isSamePath("C:\\gamedata", "C:\\gamedata-anomaly")).toBe(false);
  });
});

describe("truncatePathHead", () => {
  it("leaves a path that fits", () => {
    expect(truncatePathHead("C:\\gamedata", 40)).toBe("C:\\gamedata");
  });

  it("cuts the front, because the tail is what tells two paths apart", () => {
    const left: string = truncatePathHead("C:\\Projects\\stalker\\gamedata\\configs", 20);
    const right: string = truncatePathHead("C:\\Projects\\stalker\\gamedata-anomaly\\configs", 20);

    expect(left.startsWith("…")).toBe(true);
    expect(left).not.toBe(right);
  });

  it("keeps whole segments, so a name is never shown cut in half", () => {
    expect(truncatePathHead("C:\\Projects\\stalker\\gamedata\\configs", 20)).toBe("…gamedata\\configs");
  });

  it("cuts inside a segment that has no separator to break at", () => {
    const cut: string = truncatePathHead("averyverylongsinglesegmentname", 10);

    expect(cut).toHaveLength(10);
    expect(cut.startsWith("…")).toBe(true);
  });
});
