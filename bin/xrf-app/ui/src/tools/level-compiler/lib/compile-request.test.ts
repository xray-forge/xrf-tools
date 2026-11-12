import { describe, expect, it } from "@jest/globals";
import { ELevelCompileStep, LevelCompileStep } from "@xrf/ipc/types/xrf-level-compiler";

import { getCompileProblem, ICompileChoice, toCompileRequest } from "./compile-request";

function choice(steps: Array<LevelCompileStep>, patch: Partial<ICompileChoice> = {}): ICompileChoice {
  return {
    steps: new Set(steps),
    sdk: "C:\\SDK\\editors",
    gamedata: "C:\\Games\\CoP\\gamedata",
    textures: "C:\\Games\\CoP\\gamedata\\textures",
    ...patch,
  };
}

describe("toCompileRequest", () => {
  it("builds alone where no step is chosen, whatever else is named", () => {
    expect(toCompileRequest(choice([]))).toBeNull();
    expect(getCompileProblem(choice([], { sdk: null }))).toBeNull();
  });

  it("asks for the steps in the order the compile runs them", () => {
    const request = toCompileRequest(choice([ELevelCompileStep.AI, ELevelCompileStep.GEOMETRY]));

    expect(request?.steps).toEqual([ELevelCompileStep.GEOMETRY, ELevelCompileStep.AI]);
    expect(request?.sdk).toBe("C:\\SDK\\editors");
    expect(request?.textures).toEqual(["C:\\Games\\CoP\\gamedata\\textures"]);
  });

  it("lights the details without the SDK, which only the geometry and the AI map need", () => {
    const request = toCompileRequest(choice([ELevelCompileStep.DETAILS], { sdk: null }));

    expect(request?.steps).toEqual([ELevelCompileStep.DETAILS]);
    expect(request?.sdk).toBeNull();
    expect(getCompileProblem(choice([ELevelCompileStep.DETAILS, ELevelCompileStep.AI], { sdk: null }))).toContain(
      "SDK"
    );
  });

  it("names what a chosen step lacks before it can run", () => {
    expect(getCompileProblem(choice([ELevelCompileStep.GEOMETRY], { sdk: null }))).toContain("SDK");
    expect(getCompileProblem(choice([ELevelCompileStep.DETAILS], { gamedata: null }))).toContain("game data");
    expect(getCompileProblem(choice([ELevelCompileStep.DETAILS], { textures: null }))).toContain("textures");
    expect(toCompileRequest(choice([ELevelCompileStep.DETAILS], { textures: null }))).toBeNull();
  });
});
