import { IArchiveEntry } from "@/core/archive/lib/entry";
import { IPathDirectoryTreeItem, IPathFileTreeItem, IPathTreeItem, parsePathTree } from "@/core/ui/tree/path-tree";

/**
 * Whether a browsed file would be written when its directory is extracted.
 *
 * Mirrors `ArchiveUnpacker::extract_directory` and `XrayWorldExtractor::extract_directory` on the rust side, including
 * the skip both share: what neither writes is what `isDirectory` marks, so this reads the flag rather than guessing
 * from the name and size a second time. Counting with a plain `startsWith` here instead would promise more files than
 * the backend writes, and would let `configs` swallow `configs_backup`.
 *
 * @param descriptor - Browsed entry to test.
 * @param prefix - Engine directory path to match.
 * @returns Whether extraction would write the file under the directory.
 */
export function isUnderArchiveDirectory(descriptor: IArchiveEntry, prefix: string): boolean {
  if (descriptor.isDirectory) {
    return false;
  }

  if (!prefix) {
    return true;
  }

  const name: string = descriptor.name.toLowerCase();
  const normalized: string = prefix.replace(/[\\/]+$/, "").toLowerCase();

  return name.length > normalized.length && name.startsWith(normalized) && /[\\/]/.test(name[normalized.length]);
}

/** An archive directory node. */
export type IArchiveDirectoryTreeItem = IPathDirectoryTreeItem<IArchiveEntry>;

/** An archive file leaf, carrying the entry it was built from as its payload. */
export type IArchiveFileTreeItem = IPathFileTreeItem<IArchiveEntry>;

export type IArchiveTreeItem = IPathTreeItem<IArchiveEntry>;

/**
 * Build a directory-first explorer tree from the files a subject holds.
 *
 * The splitting, node paths and sort order come from the shared path tree; what is archive-specific is only that an
 * entry is identified by its `name` and carries itself as the payload — which is why one tree serves both a volume
 * set and a mounted world.
 *
 * @param files - Browsed entries to attach to leaf nodes.
 * @param separator - Separator used by the engine paths.
 * @returns Sorted root-level tree items with entries attached to file leaves.
 */
export function parseTree(files: Array<IArchiveEntry>, separator: string): Array<IArchiveTreeItem> {
  return parsePathTree(
    files.map((entry: IArchiveEntry) => ({ path: entry.name, payload: entry })),
    separator
  );
}
