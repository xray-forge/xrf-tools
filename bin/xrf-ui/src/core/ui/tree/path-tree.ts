import { ITreeNode } from "@/core/ui/tree/tree-node";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * Prefixes the node ids carry, so a selection says which kind of node it is.
 *
 * Owned here because this module mints the ids: a consumer that spelled them itself would be re-deriving a format it
 * does not control, which is how the archive menu ended up matching `"file:"` in four places.
 */
export const TREE_ITEM_ID = {
  directory: "directory:",
  file: "file:",
} as const;

/** Id of the synthetic root, which stands for the whole tree rather than for a path. */
export const TREE_ROOT_ID: string = `${TREE_ITEM_ID.directory}~`;

/** Separator of the engine paths these trees are built from: `\`-separated, as every X-Ray logical path is. */
export const LOGICAL_PATH_SEPARATOR: string = "\\";

/** A logical path split where a reader needs it: the file's own name, and the directories standing above it. */
export interface ILogicalPathParts {
  /** The last segment, which is the file name. */
  name: string;
  /** Everything above it, or null when the path names a file at the root. */
  directory: Nullable<string>;
}

/**
 * Splits a logical path into the part a reader identifies it by and the part that only places it.
 *
 * One policy rather than three: a search row shows the name loudly and the directory quietly, and a save dialog offers
 * the name alone. Every one of those was splitting on its own `lastIndexOf` before, which is how a separator ends up
 * spelled in four files and corrected in three.
 *
 * @param path - Engine logical path, `\`-separated.
 * @returns The file name, and the directories above it when there are any.
 */
export function splitLogicalPath(path: string): ILogicalPathParts {
  const separatorAt: number = path.lastIndexOf(LOGICAL_PATH_SEPARATOR);

  return separatorAt === -1
    ? { directory: null, name: path }
    : { directory: path.slice(0, separatorAt), name: path.slice(separatorAt + 1) };
}

/**
 * The id a leaf carries for a path.
 *
 * @param path - Path the leaf was built from.
 * @returns The node id, matching what `parsePathTree` produced.
 */
export function toFileItemId(path: string): string {
  return `${TREE_ITEM_ID.file}${path}`;
}

/**
 * The id a directory node carries for a path.
 *
 * An empty path is the synthetic root, which is how the whole tree is spelled.
 *
 * @param path - Path the directory was built from, or an empty string for the root.
 * @returns The node id, matching what `parsePathTree` produced.
 */
export function toDirectoryItemId(path: string): string {
  return path ? `${TREE_ITEM_ID.directory}${path}` : TREE_ROOT_ID;
}

/**
 * The path a node id names, when the id belongs to a leaf.
 *
 * @param itemId - Node id reported by a selection, or null when nothing is selected.
 * @returns The path, or null when the id names a directory rather than a file.
 */
export function getFileItemPath(itemId: Nullable<string>): Nullable<string> {
  return itemId?.startsWith(TREE_ITEM_ID.file) ? itemId.slice(TREE_ITEM_ID.file.length) : null;
}

/**
 * The path a node id names, when the id belongs to a directory.
 *
 * The synthetic root answers an empty path, which is how the whole tree is spelled.
 *
 * @param itemId - Node id reported by a selection, or null when nothing is selected.
 * @returns The path, or null when the id names a file rather than a directory.
 */
export function getDirectoryItemPath(itemId: Nullable<string>): Nullable<string> {
  if (!itemId?.startsWith(TREE_ITEM_ID.directory)) {
    return null;
  }

  const path: string = itemId.slice(TREE_ITEM_ID.directory.length);

  return path === "~" ? "" : path;
}

/**
 * The ids of every directory standing above a node, outermost first.
 *
 * What a reveal has to expand before a node it did not choose by hand can be seen. A node whose id encodes no path -
 * a bone, say - stands under nothing this can name, and answers an empty list.
 *
 * @param itemId - Node id to walk up from.
 * @returns Directory node ids from the root down to the immediate parent.
 */
export function getAncestorDirectoryIds(itemId: Nullable<string>): Array<string> {
  const path: Nullable<string> = getFileItemPath(itemId) ?? getDirectoryItemPath(itemId);

  if (!path) {
    return [];
  }

  const segments: Array<string> = path.split(LOGICAL_PATH_SEPARATOR);

  // The node itself is not its own ancestor, so the last segment never forms an id.
  return segments
    .slice(0, -1)
    .map((_: string, index: number) => toDirectoryItemId(segments.slice(0, index + 1).join(LOGICAL_PATH_SEPARATOR)));
}

/** A directory node, holding whatever the caller's leaves carry. */
export interface IPathDirectoryTreeItem<T> extends ITreeNode<T> {
  id: string;
  label: string;
  path: string;
  kind: "directory";
  children: Array<IPathTreeItem<T>>;
}

/** A leaf node, paired with the payload the caller identified it by. */
export interface IPathFileTreeItem<T> extends ITreeNode<T> {
  id: string;
  label: string;
  path: string;
  kind: "file";
  payload: T;
}

export type IPathTreeItem<T> = IPathDirectoryTreeItem<T> | IPathFileTreeItem<T>;

/**
 * Build a directory-first explorer tree from separated paths.
 *
 * Generic over the leaf payload because two surfaces build the same tree out of different things — archive entries and
 * roots assets — and the splitting, the canonical node paths and the sort order are the parts neither should own.
 *
 * @param entries - Paths to attach, each with the payload its leaf carries.
 * @param separator - Separator used by the paths, and used to rebuild each canonical node path.
 * @returns Sorted root-level tree items, directories before files.
 */
export function parsePathTree<T>(
  entries: Array<{ path: string; payload: T }>,
  separator: string
): Array<IPathTreeItem<T>> {
  const root: IPathDirectoryTreeItem<T> = {
    id: TREE_ROOT_ID,
    label: "root",
    path: "",
    kind: "directory",
    children: [],
  };

  for (const entry of entries) {
    appendPath(root, entry.path.split(separator), entry.payload, separator);
  }

  sortTree(root.children);

  return root.children;
}

/**
 * Append one path to a mutable directory tree.
 *
 * @param parent - Directory node that receives the next path segment.
 * @param remainingPath - Mutable path segments still to consume.
 * @param payload - Payload attached to the resulting leaf node.
 * @param separator - Separator used to reconstruct each canonical node path.
 */
function appendPath<T>(
  parent: IPathDirectoryTreeItem<T>,
  remainingPath: Array<string>,
  payload: T,
  separator: string
): void {
  const name: Optional<string> = remainingPath.shift();

  if (!name) {
    return;
  }

  const path: string = parent.path ? `${parent.path}${separator}${name}` : name;

  if (!remainingPath.length) {
    parent.children.push({ id: toFileItemId(path), label: name, path, kind: "file", payload });

    return;
  }

  const existing: Optional<IPathTreeItem<T>> = parent.children.find(
    (child: IPathTreeItem<T>) => child.kind === "directory" && child.label === name
  );
  const directory: IPathDirectoryTreeItem<T> =
    existing?.kind === "directory"
      ? existing
      : { id: `${TREE_ITEM_ID.directory}${path}`, label: name, path, kind: "directory", children: [] };

  if (!existing) {
    parent.children.push(directory);
  }

  appendPath(directory, remainingPath, payload, separator);
}

/**
 * Sort a mutable tree recursively with directories before files and labels in locale order.
 *
 * @param items - Tree items to sort in place.
 */
function sortTree<T>(items: Array<IPathTreeItem<T>>): void {
  for (const item of items) {
    if (item.kind === "directory") {
      sortTree(item.children);
    }
  }

  items.sort((first: IPathTreeItem<T>, second: IPathTreeItem<T>) => {
    if (first.kind !== second.kind) {
      return first.kind === "directory" ? -1 : 1;
    }

    return first.label.localeCompare(second.label);
  });
}
