import { SelectedSceneDescription } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayRootKind, XrayRoot, XrayRootKind } from "@xrf/ipc/types/xrf-vfs";
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
  /** Where the scene's library objects are looked for, as it was opened with; empty for `objects` beside it. */
  objects: Array<string>;
  /**
   * The game data the scene is drawn with other than the one beside its `rawdata`: the root itself, or an
   * installation's `gamedata`; none where it names no other.
   */
  gamedata: Nullable<string>;
  /** That game data's `gamemtl.xr`. */
  gameMaterials: Nullable<string>;
}

/**
 * The first game root the scene is drawn with other than the gamedata beside its `rawdata`, which the build's game
 * data comes from.
 *
 * @param scene - The open scene.
 * @returns That root's path, or none.
 */
export function getLevelEditorGameRoot(scene: SelectedSceneDescription): Nullable<string> {
  const beside: string = getBesideGamedata(scene.source.path);

  return (
    scene.game.roots.roots.map((root: XrayRoot) => root.path).find((root: string) => !isSamePath(root, beside)) ?? null
  );
}

/**
 * Where the level compiler builds the scene the editor shows, as a decompile lays a scene out: the scene in
 * `rawdata\levels`, its build and the textures its decompile cut in `gamedata` beside `rawdata`, its objects where it
 * was opened with them, and the game's own data from the game root the scene is drawn with.
 *
 * @param scene - The open scene.
 * @param gameRootKind - What the game root ([`getLevelEditorGameRoot`]) is, so an installation hands its `gamedata`;
 *   none where it is unknown, the root taken as game data.
 * @returns The compiler form's paths.
 */
export function getLevelEditorBuildPaths(
  scene: SelectedSceneDescription,
  gameRootKind: Nullable<XrayRootKind>
): ILevelEditorBuildPaths {
  const path: string = scene.source.path;
  const separator: string = getPathSeparator(path);
  const beside: string = getBesideGamedata(path);
  const root: Nullable<string> = getLevelEditorGameRoot(scene);
  const gamedata: Nullable<string> =
    root === null ? null : gameRootKind === EXrayRootKind.INSTALLATION ? joinPath(root, "gamedata") : root;

  return {
    source: path,
    destination: [beside, "levels", scene.name].join(separator),
    textures: [beside, "textures"].join(separator),
    objects: scene.objectsRoots.roots.map((objects: XrayRoot) => objects.path),
    gamedata,
    gameMaterials: gamedata === null ? null : joinPath(gamedata, "gamemtl.xr"),
  };
}

/** The `gamedata` beside the `rawdata` holding the scene's `levels` directory. */
function getBesideGamedata(path: string): string {
  return [getPathDirectory(getPathDirectory(getPathDirectory(path))), "gamedata"].join(getPathSeparator(path));
}

function joinPath(directory: string, name: string): string {
  return [directory.replace(/[\\/]+$/, ""), name].join(getPathSeparator(directory));
}
