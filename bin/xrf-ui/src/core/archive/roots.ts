import { createRoot } from "@/core/assets/lib";
import { ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/**
 * The roots an archived asset is read out of, which is the project's own tree.
 *
 * Centred on nothing: an entry has no filesystem path of its own to search beside, so the volumes under the project
 * root are the whole roots.
 *
 * Read as `volumes` rather than the default `auto`. A project is opened by reading its root recursively, while `auto`
 * mounts only the volumes a directory holds directly - what an `fsgame.ltx` alias declares. Left on the default, every
 * entry stored in a volume inside a subdirectory listed and extracted but previewed as missing.
 *
 * @param project - Opened archive project to read from.
 * @returns The roots spec every read of this project's entries names.
 */
export function createArchiveRoots(project: ArchiveProject): XrayRoots {
  return { asset: null, roots: [createRoot(project.root, "volumes")] };
}
