import { describe, expect, it } from "@jest/globals";

import {
  findLastSeparator,
  getPathName,
  LOGICAL_PATH_SEPARATOR,
  splitAfterSeparators,
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
