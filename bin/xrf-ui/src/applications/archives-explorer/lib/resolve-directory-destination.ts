import * as path from "@tauri-apps/api/path";
import { LOGICAL_PATH_SEPARATOR, splitLogicalPath } from "@xrf/path";

/**
 * Where extracting one archive directory writes, given the folder picked for it.
 *
 * The directory lands as itself inside the picked folder, the way copying a folder does: `configs\gameplay` picked
 * into `C:\out` writes `C:\out\gameplay\...`, without the `configs` parents above it. The tree root has no name of its
 * own, so it writes into the picked folder directly.
 *
 * @param picked - Host directory the person chose.
 * @param prefix - Archive directory being extracted; empty for the tree root.
 * @returns Host directory the extraction should write into.
 */
export async function resolveDirectoryDestination(picked: string, prefix: string): Promise<string> {
  const name: string = splitLogicalPath(prefix.replaceAll("/", LOGICAL_PATH_SEPARATOR).replace(/\\+$/, "")).name;

  return name ? path.join(picked, name) : picked;
}
