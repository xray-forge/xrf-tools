import { describe, expect, it } from "@jest/globals";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import { EXrayRootKind } from "@xrf/ipc/types/xrf-vfs";

import { createRoots } from "@/core/assets/lib/roots";
import { mockSelectedSceneDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import {
  getLevelEditorBuildPaths,
  getLevelEditorGameRoot,
} from "@/tools/level-editor/lib/build/level-editor-build-paths";

describe("getLevelEditorBuildPaths", () => {
  it("builds a decompiled scene into the gamedata beside its rawdata, with the game it is drawn with", () => {
    const paths = getLevelEditorBuildPaths(
      mockSelectedSceneDescription({
        game: {
          engine: EXrayEngineChoice.AUTO,
          isDltx: false,
          roots: createRoots(["D:\\qa\\labx8\\gamedata", "C:\\Games\\CoP\\gamedata\\"]),
        },
        objectsRoots: createRoots(["D:\\sdk\\rawdata\\objects", "D:\\qa\\labx8\\rawdata\\objects"]),
        source: { path: "D:\\qa\\labx8\\rawdata\\levels\\labx8.level" },
      }),
      EXrayRootKind.ROOT
    );

    expect(paths).toEqual({
      source: "D:\\qa\\labx8\\rawdata\\levels\\labx8.level",
      destination: "D:\\qa\\labx8\\gamedata\\levels\\labx8",
      textures: "D:\\qa\\labx8\\gamedata\\textures",
      objects: ["D:\\sdk\\rawdata\\objects", "D:\\qa\\labx8\\rawdata\\objects"],
      gamedata: "C:\\Games\\CoP\\gamedata\\",
      gameMaterials: "C:\\Games\\CoP\\gamedata\\gamemtl.xr",
    });
  });

  // An installation holds its libraries in gamedata\, not at its top, where the compilers would not find them.
  it("hands an installation's gamedata as the game data", () => {
    const paths = getLevelEditorBuildPaths(
      mockSelectedSceneDescription({
        game: { engine: EXrayEngineChoice.AUTO, isDltx: false, roots: createRoots(["C:\\Games\\CoP\\"]) },
        source: { path: "D:\\qa\\labx8\\rawdata\\levels\\labx8.level" },
      }),
      EXrayRootKind.INSTALLATION
    );

    expect(paths.gamedata).toBe("C:\\Games\\CoP\\gamedata");
    expect(paths.gameMaterials).toBe("C:\\Games\\CoP\\gamedata\\gamemtl.xr");
  });

  it("names no game data where the scene is drawn only with the gamedata beside its rawdata, or none", () => {
    const scene = mockSelectedSceneDescription({
      game: { engine: EXrayEngineChoice.AUTO, isDltx: false, roots: createRoots(["d:/qa/labx8/GAMEDATA/"]) },
      source: { path: "D:/qa/labx8/rawdata/levels/labx8.level" },
    });
    const beside = getLevelEditorBuildPaths(scene, null);

    expect(getLevelEditorGameRoot(scene)).toBeNull();
    expect(beside.destination).toBe("D:/qa/labx8/gamedata/levels/labx8");
    expect(beside.gamedata).toBeNull();
    expect(beside.gameMaterials).toBeNull();
    expect(getLevelEditorBuildPaths(mockSelectedSceneDescription(), null).gamedata).toBeNull();
  });
});
