import { describe, expect, it } from "@jest/globals";
import { SpawnListedLevel } from "@xrf/ipc/types/xrf-spawn-builder";

import { getSpawnBuildProblem, ISpawnBuildChoice, toggleSpawnLevel, toSpawnBuildRequest } from "./spawn-request";

function choice(patch: Partial<ISpawnBuildChoice> = {}): ISpawnBuildChoice {
  return {
    gamedata: "D:\\qa\\gamedata",
    levelsRoot: "D:\\qa\\gamedata\\levels",
    destination: "D:\\qa\\spawn",
    levels: ["zaton", "labx8"],
    start: "labx8",
    name: "qa_levels",
    isSeparatorStrict: false,
    isForced: false,
    ...patch,
  };
}

function level(name: string, id: number): SpawnListedLevel {
  return { name, caption: name, id, isComplete: true };
}

describe("toSpawnBuildRequest", () => {
  it("asks for the build `spawn build` runs over the levels ticked, into the spawn directory named", () => {
    expect(toSpawnBuildRequest(choice({ name: " qa_levels " }))).toEqual({
      gamedata: "D:\\qa\\gamedata",
      levelsRoot: "D:\\qa\\gamedata\\levels",
      levels: ["zaton", "labx8"],
      destination: "D:\\qa\\spawn",
      name: "qa_levels",
      options: { start: "labx8", isSeparatorStrict: false },
      isForced: false,
    });
  });

  it("leaves the actor where it stands where no start level is chosen", () => {
    expect(toSpawnBuildRequest(choice({ start: null }))?.options.start).toBeNull();
  });

  it("asks for nothing while a path is missing", () => {
    expect(toSpawnBuildRequest(choice({ gamedata: null }))).toBeNull();
    expect(toSpawnBuildRequest(choice({ levelsRoot: null }))).toBeNull();
    expect(toSpawnBuildRequest(choice({ destination: null }))).toBeNull();
  });
});

describe("getSpawnBuildProblem", () => {
  it("names the row to change: no level, a start level not ticked, a name that is no file name", () => {
    expect(getSpawnBuildProblem(choice({ levels: [] }))?.field).toBe("levels");
    expect(getSpawnBuildProblem(choice({ levels: ["zaton"] }))?.field).toBe("start");

    for (const name of ["", "  ", "spawns\\qa", "qa/levels", "..", "qa:levels"]) {
      expect(getSpawnBuildProblem(choice({ name }))?.field).toBe("name");
      expect(toSpawnBuildRequest(choice({ name }))).toBeNull();
    }
  });

  it("leaves writing over a spawn already there to the build, which sees whether it is", () => {
    expect(getSpawnBuildProblem(choice({ name: "all" }))).toBeNull();
    expect(toSpawnBuildRequest(choice({ name: "all", isForced: true }))?.isForced).toBe(true);
  });
});

describe("toggleSpawnLevel", () => {
  const listed: Array<SpawnListedLevel> = [level("zaton", 108), level("jupiter", 109), level("labx8", 150)];

  it("keeps the levels in the order the game lists them, whatever order they were ticked in", () => {
    const ticked = toggleSpawnLevel(listed, toggleSpawnLevel(listed, [], "labx8", true), "zaton", true);

    expect(ticked).toEqual(["zaton", "labx8"]);
    expect(toggleSpawnLevel(listed, ticked, "labx8", false)).toEqual(["zaton"]);
  });

  it("drops a ticked level the listing no longer offers", () => {
    expect(toggleSpawnLevel(listed, ["pripyat", "zaton"], "jupiter", true)).toEqual(["zaton", "jupiter"]);
  });
});
