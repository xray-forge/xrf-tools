import { SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { XrayRoot } from "@xrf/ipc/types/xrf-vfs";
import { getPathDirectory, getPathSeparator, isSamePath } from "@xrf/path";
import { Nullable } from "@xrf/types";

/** The level compiler's paths for building the scene the editor shows. */
export interface ILevelEditorBuildPaths {
  /** The scene's `.level`. */
  source: string;
  /** `gamedata\levels\<name>` beside the scene's `rawdata`, where an SDK keeps a scene's build. */
  destination: string;
  /**
   * `gamedata\textures` beside the scene's `rawdata`, where its decompile writes the textures it cuts: the build's
   * impostor strips and details, read before the game data's own.
   */
  textures: string;
  /** The game data the scene is drawn with other than the one beside its `rawdata`, none where it names no other. */
  gamedata: Nullable<string>;
  /** That game data's `gamemtl.xr`. */
  gameMaterials: Nullable<string>;
}

/**
 * Where the level compiler builds the scene the editor shows, as a decompile lays a scene out: the scene in
 * `rawdata\levels`, its build and the textures its decompile cut in `gamedata` beside `rawdata`, and the game's own data
 * the game root the scene is drawn with.
 *
 * @param scene - The open scene.
 * @returns The compiler form's paths.
 */
export function getLevelEditorBuildPaths(scene: SelectedSceneDescription): ILevelEditorBuildPaths {
  const path: string = scene.source.path;
  const separator: string = getPathSeparator(path);
  const beside: string = [getPathDirectory(getPathDirectory(getPathDirectory(path))), "gamedata"].join(separator);
  const game: Nullable<string> =
    scene.game.roots.roots.map((root: XrayRoot) => root.path).find((root: string) => !isSamePath(root, beside)) ?? null;

  return {
    source: path,
    destination: [beside, "levels", scene.name].join(separator),
    textures: [beside, "textures"].join(separator),
    gamedata: game,
    gameMaterials: game === null ? null : [game.replace(/[\\/]+$/, ""), "gamemtl.xr"].join(getPathSeparator(game)),
  };
}
