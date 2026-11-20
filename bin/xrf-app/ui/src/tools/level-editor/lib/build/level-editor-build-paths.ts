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
   * `gamedata\textures` beside the scene's `rawdata`, where a decompile writes impostor strips and details, read before
   * the game data's own.
   */
  textures: string;
  /** The objects roots the scene was opened with; empty for `objects` beside it. */
  objects: Array<string>;
  /**
   * The game data from the scene's other game root: the root itself, or an installation's `gamedata`; null where it
   * names no other.
   */
  gamedata: Nullable<string>;
  /** That game data's `gamemtl.xr`. */
  gameMaterials: Nullable<string>;
}

/**
 * The scene's first game root other than the `gamedata` beside its `rawdata`, which the build's game data comes from.
 */
export function getLevelEditorGameRoot(scene: SelectedSceneDescription): Nullable<string> {
  const beside: string = getBesideGamedata(scene.source.path);

  return (
    scene.game.roots.roots.map((root: XrayRoot) => root.path).find((root: string) => !isSamePath(root, beside)) ?? null
  );
}

/**
 * The level compiler form's paths for the open scene, laid out as a decompile writes a scene.
 *
 * @param scene - The open scene.
 * @param gameRootKind - The kind of {@link getLevelEditorGameRoot}'s root, so an installation hands its `gamedata`;
 *   null where unknown, the root then taken as game data.
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
