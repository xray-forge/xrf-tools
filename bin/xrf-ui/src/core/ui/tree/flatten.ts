import { IPathTreeItem } from "@/core/ui/tree/path-tree";

/**
 * One visible row of a flattened tree.
 *
 * Virtualization renders a window of siblings rather than nested lists, so everything the nesting used to
 * convey - depth, whether a node opens, and its position among its siblings - has to travel on the row.
 */
export interface IFlatTreeRow<T> {
  item: IPathTreeItem<T>;
  /** Nesting level, zero at the root, used for indentation and `aria-level`. */
  depth: number;
  /** Whether the node has children to reveal, which is what earns it a chevron. */
  hasChildren: boolean;
  /** Whether those children are currently revealed. */
  isExpanded: boolean;
  /** Number of siblings at this level, for `aria-setsize`. */
  setSize: number;
  /** One-based position among those siblings, for `aria-posinset`. */
  posInSet: number;
  /** Id of the enclosing directory, so a collapse can move selection to the parent. */
  parentId: string | null;
}

/**
 * Flatten the visible part of a tree, depth first, in the order rows appear on screen.
 *
 * Only expanded directories contribute children, so the result is exactly the rows a fully rendered tree
 * would show, and its length is the row count the virtualizer scrolls through.
 *
 * @param items - Root level tree items.
 * @param expandedIds - Ids of the directories whose children are revealed.
 * @returns Visible rows in display order.
 */
export function flattenTree<T>(
  items: Array<IPathTreeItem<T>>,
  expandedIds: ReadonlySet<string>
): Array<IFlatTreeRow<T>> {
  const rows: Array<IFlatTreeRow<T>> = [];

  function walk(siblings: Array<IPathTreeItem<T>>, depth: number, parentId: string | null): void {
    for (let index = 0; index < siblings.length; index += 1) {
      const item: IPathTreeItem<T> = siblings[index];
      const hasChildren: boolean = item.kind === "directory" && item.children.length > 0;
      const isExpanded: boolean = hasChildren && expandedIds.has(item.id);

      rows.push({
        item,
        depth,
        hasChildren,
        isExpanded,
        setSize: siblings.length,
        posInSet: index + 1,
        parentId,
      });

      if (isExpanded && item.kind === "directory") {
        walk(item.children, depth + 1, item.id);
      }
    }
  }

  walk(items, 0, null);

  return rows;
}
