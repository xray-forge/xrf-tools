import { Box } from "@mui/material";
import { LayoutList, useVirtualizer } from "@mui/x-virtualizer";
import { KeyboardEvent, ReactElement, ReactNode, useCallback, useEffect, useId, useMemo, useRef } from "react";

import { TREE } from "@/core/theme/tokens";
import { flattenTree, IFlatTreeRow } from "@/core/ui/tree/flatten";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { VirtualizedTreeRow } from "@/core/ui/tree/VirtualizedTree/VirtualizedTreeRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** Icons the tree draws beside the chevron, chosen per row by what the row is. */
export interface IVirtualizedTreeIcons {
  /** A node with children that is closed. */
  collapsed: ReactNode;
  /** A node with children that is open. */
  expanded: ReactNode;
  /** A leaf. */
  leaf: ReactNode;
}

export interface IVirtualizedTreeProps<T> extends BaseComponentProps {
  items: ReadonlyArray<ITreeNode<T>>;
  expandedIds: ReadonlySet<string>;
  selectedId: Nullable<string>;
  ariaLabel: string;
  /** Omitted by a tree whose rows are all one kind of thing, which then draws the chevron alone. */
  icons?: IVirtualizedTreeIcons;
  /** Decorates a row's label, for a consumer that marks where the entry came from. */
  renderLabel?: (item: ITreeNode<T>) => ReactNode;
  onToggleExpanded: (id: string) => void;
  /** A row was chosen for inspection, by click or by arrow key. */
  onSelect: (item: ITreeNode<T>) => void;
  /** A row was activated, by double click or by `Enter`. */
  onActivate: (item: ITreeNode<T>) => void;
}

/**
 * A tree that renders only the rows on screen.
 *
 * Cost is the size of the viewport rather than the size of the directory, which is the whole point: MUI's
 * `RichTreeView` renders every revealed row, and a directory of two thousand entries costs a third of a
 * second to open. Virtualizing requires a flat DOM - a window of rows cannot be sliced out of nested
 * lists - so this owns the tree semantics that nesting used to provide.
 */
export function VirtualizedTree<T>({
  "data-testid": dataTestId = "virtualized-tree",
  id,
  className,
  sx,
  items,
  expandedIds,
  selectedId,
  ariaLabel,
  icons,
  renderLabel,
  onToggleExpanded,
  onSelect,
  onActivate,
}: IVirtualizedTreeProps<T>): ReactElement {
  const treeId: string = useId();

  const rows: Array<IFlatTreeRow<T>> = useMemo(() => flattenTree(items, expandedIds), [items, expandedIds]);

  const scrollerRef = useRef<HTMLElement | null>(null);
  const layoutRef = useRef<Nullable<LayoutList>>(null);

  if (!layoutRef.current) {
    // The constructor takes these, but `LayoutList.use` reads the refs it is handed in `layoutParams`
    // instead, so nothing ever writes to them. The scroller node is captured below rather than here.
    layoutRef.current = new LayoutList({ container: { current: null }, scroller: { current: null } });
  }

  const virtualizerRows = useMemo(() => rows.map((row: IFlatTreeRow<T>) => ({ id: row.item.id, model: row })), [rows]);

  const range = useMemo(() => ({ firstRowIndex: 0, lastRowIndex: rows.length }), [rows.length]);

  const rowIdOf = useCallback((index: number) => `${treeId}-row-${index}`, [treeId]);

  // A selection that no longer exists - its directory closed, its listing replaced - simply draws nothing, and
  // the first arrow key lands on the first row.
  const selectedIndex: number = useMemo(
    () => (selectedId ? rows.findIndex((row: IFlatTreeRow<T>) => row.item.id === selectedId) : -1),
    [rows, selectedId]
  );

  const activate = useCallback(
    (row: IFlatTreeRow<T>) => {
      // Opening a node with children is opening the node: the consumer hears about it either way, and decides
      // for itself whether standing on a directory means anything.
      if (row.hasChildren) {
        onToggleExpanded(row.item.id);
      }

      onActivate(row.item);
    },
    [onActivate, onToggleExpanded]
  );

  const virtualizer = useVirtualizer({
    layout: layoutRef.current,
    dimensions: { rowHeight: TREE.rowHeight },
    virtualization: {},
    rows: virtualizerRows,
    range,
    rowCount: rows.length,
    renderRow: (params) => {
      const row: IFlatTreeRow<T> = params.model as unknown as IFlatTreeRow<T>;
      const icon: Nullable<ReactNode> = icons
        ? row.hasChildren
          ? row.isExpanded
            ? icons.expanded
            : icons.collapsed
          : icons.leaf
        : null;

      return (
        <VirtualizedTreeRow<T>
          key={row.item.id}
          row={row}
          rowId={rowIdOf(params.rowIndex)}
          icon={icon}
          isSelected={row.item.id === selectedId}
          label={renderLabel?.(row.item)}
          onSelect={(it: IFlatTreeRow<T>) => onSelect(it.item)}
          onActivate={activate}
          onToggleExpanded={onToggleExpanded}
        />
      );
    },
  });

  const containerProps = virtualizer.store.use(LayoutList.selectors.containerProps);
  const contentProps = virtualizer.store.use(LayoutList.selectors.contentProps);
  const positionerProps = virtualizer.store.use(LayoutList.selectors.positionerProps);

  /**
   * Hands the scroller node to the virtualizer and keeps a reference to it.
   *
   * The virtualizer's own ref is what attaches its scroll and resize listeners, so it has to be called;
   * keeping the node as well is what lets the keyboard scroll a row into view.
   */
  const setScroller = useCallback(
    (node: Nullable<HTMLElement>): void => {
      scrollerRef.current = node;

      const attach: unknown = containerProps.ref;

      if (typeof attach === "function") {
        (attach as (element: Nullable<HTMLElement>) => void)(node);
      } else if (attach) {
        (attach as { current: Nullable<HTMLElement> }).current = node;
      }
    },
    [containerProps.ref]
  );

  /** Scrolls a row into view by arithmetic, since every row is exactly one `TREE.rowHeight` tall. */
  const revealRow = useCallback((index: number): void => {
    const scroller: Nullable<HTMLElement> = scrollerRef.current;

    if (!scroller || index < 0) {
      return;
    }

    const top: number = index * TREE.rowHeight;
    const bottom: number = top + TREE.rowHeight;

    if (top < scroller.scrollTop) {
      scroller.scrollTop = top;
    } else if (bottom > scroller.scrollTop + scroller.clientHeight) {
      scroller.scrollTop = bottom - scroller.clientHeight;
    }
  }, []);

  const moveTo = useCallback(
    (index: number): void => {
      const next: number = Math.min(Math.max(index, 0), rows.length - 1);
      const row: Nullable<IFlatTreeRow<T>> = rows[next] ?? null;

      if (row) {
        onSelect(row.item);
        revealRow(next);
      }
    },
    [onSelect, revealRow, rows]
  );

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>): void => {
      const row: Nullable<IFlatTreeRow<T>> = rows[selectedIndex] ?? null;

      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();

          return moveTo(selectedIndex + 1);

        case "ArrowUp":
          event.preventDefault();

          return moveTo(selectedIndex - 1);

        case "Home":
          event.preventDefault();

          return moveTo(0);

        case "End":
          event.preventDefault();

          return moveTo(rows.length - 1);

        case "ArrowRight": {
          event.preventDefault();

          if (!row) {
            return;
          }

          // Open what is closed, and step into what is already open, which is what a tree does here.
          if (row.hasChildren && !row.isExpanded) {
            return onToggleExpanded(row.item.id);
          }

          if (row.isExpanded) {
            return moveTo(selectedIndex + 1);
          }

          return;
        }

        case "ArrowLeft": {
          event.preventDefault();

          if (!row) {
            return;
          }

          if (row.isExpanded) {
            return onToggleExpanded(row.item.id);
          }

          // Otherwise leave the directory, which is the row the flattening recorded as the parent.
          if (row.parentId) {
            const parentIndex: number = rows.findIndex((it: IFlatTreeRow<T>) => it.item.id === row.parentId);

            if (parentIndex !== -1) {
              return moveTo(parentIndex);
            }
          }

          return;
        }

        case "Enter": {
          event.preventDefault();

          if (row) {
            activate(row);
          }

          return;
        }

        default:
          return;
      }
    },
    [activate, moveTo, onToggleExpanded, rows, selectedIndex]
  );

  // Covers a selection the tree did not make: a filter result, or a session restored into a directory that had
  // to be opened before the row existed at all.
  useEffect(() => revealRow(selectedIndex), [revealRow, selectedIndex]);

  return (
    <Box
      {...containerProps}
      ref={setScroller}
      aria-activedescendant={selectedIndex === -1 ? undefined : rowIdOf(selectedIndex)}
      aria-label={ariaLabel}
      data-testid={dataTestId}
      id={id}
      className={className}
      role={"tree"}
      sx={{
        height: "100%",
        outline: "none",
        overflow: "auto",
        padding: 0.5,
        // Every tree draws its selection; the ring says which one the keyboard is talking to, a live question
        // beside a graph canvas or a viewport.
        "&:focus [aria-selected=true]": {
          outline: "1px solid",
          outlineColor: "primary.main",
          outlineOffset: "-1px",
        },
        ...sx,
      }}
      tabIndex={0}
      onKeyDown={onKeyDown}
    >
      <div {...contentProps} />
      <div {...positionerProps} role={"presentation"} />

      {virtualizer.api.getters.getRows()}
    </Box>
  );
}
