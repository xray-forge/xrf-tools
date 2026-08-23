import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { IPathDirectoryTreeItem, IPathFileTreeItem, IPathTreeItem, parsePathTree } from "@/core/ui/tree/path-tree";

/**
 * Whether an archived file would be written when its directory is extracted.
 *
 * Mirrors `ArchiveUnpacker::extract_directory` on the rust side, including its skip: what it declines to write is what
 * `isDirectory` marks, so this reads the flag rather than guessing from the name and size a second time. Counting with
 * a plain `startsWith` here instead would promise more files than the backend writes, and would let `configs` swallow
 * `configs_backup`.
 *
 * @param descriptor - Archive file metadata to test.
 * @param prefix - Archive-relative directory path to match.
 * @returns Whether extraction would write the file under the directory.
 */
export function isUnderArchiveDirectory(descriptor: ArchiveFileDescriptor, prefix: string): boolean {
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
export type IArchiveDirectoryTreeItem = IPathDirectoryTreeItem<ArchiveFileDescriptor>;

/** An archive file leaf, carrying the descriptor it was built from as its payload. */
export type IArchiveFileTreeItem = IPathFileTreeItem<ArchiveFileDescriptor>;

export type IArchiveTreeItem = IPathTreeItem<ArchiveFileDescriptor>;

/**
 * Build a directory-first explorer tree from effective archive file descriptors.
 *
 * The splitting, node paths and sort order come from the shared path tree; what is archive-specific is only that an
 * entry is identified by its `name` and carries its descriptor.
 *
 * @param files - Effective archive files to attach to leaf nodes.
 * @param separator - Separator used by the archive-relative file paths.
 * @returns Sorted root-level tree items with descriptors attached to file leaves.
 */
export function parseTree(files: Array<ArchiveFileDescriptor>, separator: string): Array<IArchiveTreeItem> {
  return parsePathTree(
    files.map((descriptor: ArchiveFileDescriptor) => ({ path: descriptor.name, payload: descriptor })),
    separator
  );
}
