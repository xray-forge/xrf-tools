import { XrayMountMode, XrayWorldRoot, XrayWorldSpec } from "@/core/bindings/types/xrf-vfs";
import { Nullable } from "@/lib/types/general";

/**
 * How a path the user picked is read.
 *
 * `auto` treats it as whatever it is: an installation with its volumes, a bare volume set, or a loose
 * tree. A viewer pointed at an installation's `db` directory otherwise mounts the volumes as files and
 * finds no assets at all.
 */
export const DEFAULT_MOUNT_MODE: XrayMountMode = "auto";

/**
 * Names a world for the backend to mount.
 *
 * Every surface says where to read from the same way, so the mode default lives here rather than being
 * repeated at each call site. Roots are searched in the order given, and the first one holding a path
 * wins — which is how a loose gamedata tree shadows the installation behind it.
 *
 * @param roots - Paths to search, highest priority first. Empty entries are dropped.
 * @param asset - Asset whose own root and installation are searched before them.
 * @returns A world spec the backend can mount.
 */
export function createWorldSpec(roots: Array<Nullable<string>>, asset: Nullable<string> = null): XrayWorldSpec {
  return {
    asset,
    roots: roots
      .filter((root: Nullable<string>): root is string => Boolean(root))
      .map((root: string) => createWorldRoot(root)),
  };
}

/**
 * Names one root, read the default way.
 *
 * @param path - Path to search.
 * @param mode - How the path is read. Defaults to `auto`.
 * @returns One root of a world spec.
 */
export function createWorldRoot(path: string, mode: XrayMountMode = DEFAULT_MOUNT_MODE): XrayWorldRoot {
  return { mode, path };
}
