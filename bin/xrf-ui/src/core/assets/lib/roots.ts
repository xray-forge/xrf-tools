import { XrayMountMode, XrayRoot, XrayRoots } from "@/core/bindings/types/xrf-vfs";
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
 * Names where the backend should read from.
 *
 * Every surface says where to read from the same way, so the mode default lives here rather than being
 * repeated at each call site. Roots are searched in the order given, and the first one holding a path
 * wins — which is how a loose gamedata tree shadows the installation behind it.
 *
 * @param roots - Paths to search, highest priority first. Empty entries are dropped.
 * @param asset - Asset whose own root and installation are searched before them.
 * @returns A roots spec the backend can mount.
 */
export function createRoots(roots: Array<Nullable<string>>, asset: Nullable<string> = null): XrayRoots {
  return {
    asset,
    roots: roots
      .filter((root: Nullable<string>): root is string => Boolean(root))
      .map((root: string) => createRoot(root)),
  };
}

/**
 * Names one root, read the default way.
 *
 * @param path - Path to search.
 * @param mode - How the path is read. Defaults to `auto`.
 * @returns One root of a declaration.
 */
export function createRoot(path: string, mode: XrayMountMode = DEFAULT_MOUNT_MODE): XrayRoot {
  return { mode, path };
}

/**
 * Names roots for a log line or a message.
 *
 * Mirrors `XrayRoots::describe` on the Rust side, because roots have no single path to print and a
 * caller joining the root objects directly gets `[object Object]`.
 *
 * @param roots - Roots to name.
 * @returns The paths, comma separated, or the subject asset when there are none.
 */
export function describeRoots(roots: XrayRoots): string {
  if (!roots.roots.length) {
    return roots.asset ?? "<no roots>";
  }

  return roots.roots.map((root: XrayRoot) => root.path).join(", ");
}
