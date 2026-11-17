import { describe, expect, it } from "@jest/globals";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { createRoots } from "@/core/assets/lib/roots";
import { mockSelectedSceneDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import { getLevelEditorBuildPaths } from "@/tools/level-editor/lib/build/level-editor-build-paths";

describe("getLevelEditorBuildPaths", () => {
  it("builds a decompiled scene into the gamedata beside its rawdata, with the game it is drawn with", () => {
    const paths = getLevelEditorBuildPaths(
      mockSelectedSceneDescription({
        game: {
          engine: EXrayEngineChoice.AUTO,
          isDltx: false,
          roots: createRoots(["D:\\qa\\labx8\\gamedata", "C:\\Games\\CoP\\gamedata\\"]),
        },
        source: { path: "D:\\qa\\labx8\\rawdata\\levels\\labx8.level" },
      })
    );

    expect(paths).toEqual({
      source: "D:\\qa\\labx8\\rawdata\\levels\\labx8.level",
      destination: "D:\\qa\\labx8\\gamedata\\levels\\labx8",
      textures: "D:\\qa\\labx8\\gamedata\\textures",
      gamedata: "C:\\Games\\CoP\\gamedata\\",
      gameMaterials: "C:\\Games\\CoP\\gamedata\\gamemtl.xr",
    });
  });

  it("names no game data where the scene is drawn only with the gamedata beside its rawdata, or none", () => {
    const beside = getLevelEditorBuildPaths(
      mockSelectedSceneDescription({
        game: { engine: EXrayEngineChoice.AUTO, isDltx: false, roots: createRoots(["d:/qa/labx8/GAMEDATA/"]) },
        source: { path: "D:/qa/labx8/rawdata/levels/labx8.level" },
      })
    );

    expect(beside.destination).toBe("D:/qa/labx8/gamedata/levels/labx8");
    expect(beside.gamedata).toBeNull();
    expect(beside.gameMaterials).toBeNull();
    expect(getLevelEditorBuildPaths(mockSelectedSceneDescription()).gamedata).toBeNull();
  });
});
