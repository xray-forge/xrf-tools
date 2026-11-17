import { describe, expect, it } from "@jest/globals";

import { getSpawnHandOverPaths } from "./spawn-hand-over-paths";

describe("getSpawnHandOverPaths", () => {
  it("builds the spawn over the level's neighbours, into spawns beside them, with the game data compiled with", () => {
    expect(getSpawnHandOverPaths("D:\\qa\\labx8\\gamedata\\levels\\labx8", "C:\\Games\\CoP\\gamedata")).toEqual({
      levelsRoot: "D:\\qa\\labx8\\gamedata\\levels",
      destination: "D:\\qa\\labx8\\gamedata\\spawns",
      gamedata: "C:\\Games\\CoP\\gamedata",
    });
  });

  it("reads a level directory written with a closing separator or forward slashes alike", () => {
    expect(getSpawnHandOverPaths("D:/qa/gamedata/levels/zaton/", null)).toEqual({
      levelsRoot: "D:/qa/gamedata/levels",
      destination: "D:/qa/gamedata/spawns",
      gamedata: null,
    });
  });
});
