import { getPathDirectory, getPathSeparator } from "@xrf/path";
import { Nullable } from "@xrf/types";

/** The level editor's open form paths for a scene a decompile wrote. */
export interface IEditorHandOverPaths {
  /** The scene's `.level`. */
  scene: string;
  /** The roots its objects are looked for in, in order; empty for `objects` beside the scene's directory. */
  objects: Array<string>;
}

/**
 * The paths the editor opens a decompiled scene with: the scene, and the roots its objects lie in. The decompile's own
 * objects come first, from the objects root it was given or `objects` beside the scene's directory where it wrote them,
 * then the libraries it placed objects from by reference. Without a library or an objects root the list stays empty,
 * which the editor reads as `objects` beside the scene, as the decompile wrote them.
 *
 * @param scene - The `.level` the decompile wrote.
 * @param objects - The objects root it wrote the level's own objects into; null for `objects` beside the scene.
 * @param libraries - The libraries it placed objects from, empty and padded entries left out.
 */
export function getEditorHandOverPaths(
  scene: string,
  objects: Nullable<string>,
  libraries: ReadonlyArray<string>
): IEditorHandOverPaths {
  const placed: Array<string> = libraries.map((library: string) => library.trim()).filter(Boolean);

  if (objects === null && placed.length === 0) {
    return { scene, objects: [] };
  }

  const own: string = objects ?? [getPathDirectory(getPathDirectory(scene)), "objects"].join(getPathSeparator(scene));

  return { scene, objects: [own, ...placed] };
}
