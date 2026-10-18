import * as path from "@tauri-apps/api/path";
import { LOGICAL_PATH_SEPARATOR, splitLogicalPath } from "@xrf/path";

/**
 * Where extracting one archive directory writes: its own folder inside the picked one, or the picked one for the root.
 *
 * @param picked - Host directory the person chose.
 * @param prefix - Archive directory being extracted; empty for the tree root.
 * @returns Host directory the extraction should write into.
 */
export async function resolveDirectoryDestination(picked: string, prefix: string): Promise<string> {
  const name: string = splitLogicalPath(prefix.replaceAll("/", LOGICAL_PATH_SEPARATOR).replace(/\\+$/, "")).name;

  return name ? path.join(picked, name) : picked;
}
