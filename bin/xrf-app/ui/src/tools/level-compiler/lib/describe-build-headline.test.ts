import { describe, expect, it } from "@jest/globals";
import { LevelCompileReport } from "@xrf/ipc/types/xrf-level-compiler";
import { LevelBuildReport } from "@xrf/ipc/types/xrf-rawdata";

import { describeBuildHeadline } from "./describe-build-headline";

const REPORT = { source: "scene.level", destination: "levels\\test" } as LevelBuildReport;
const COMPILE: LevelCompileReport = {
  level: "levels\\test",
  work: "work",
  steps: [
    { step: "geometry", program: "xrLC.exe", duration: 1000, log: null },
    { step: "details", program: "xrDO_Light.exe", duration: 500, log: null },
  ],
};

describe("describeBuildHeadline", () => {
  it("names the build alone where nothing was compiled", () => {
    expect(describeBuildHeadline(REPORT, null, false)).toBe("Built scene.level into levels\\test");
  });

  it("names the steps the compile ran", () => {
    expect(describeBuildHeadline(REPORT, COMPILE, false)).toContain("compiled it: geometry, details");
  });

  it("says a compile stopped after the build keeps what it wrote", () => {
    expect(describeBuildHeadline(REPORT, null, true)).toContain("the compile was stopped");
  });
});
