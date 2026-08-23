import { Box } from "@mui/material";
import { LayoutList, useVirtualizer } from "@mui/x-virtualizer";
import {
  KeyboardEvent,
  ReactElement,
  ReactNode,
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";

import { TREE } from "@/core/theme/tokens";
import { flattenTree, IFlatTreeRow } from "@/core/ui/tree/flatten";
import { IPathTreeItem } from "@/core/ui/tree/path-tree";
import { VirtualizedTreeRow } from "@/core/ui/tree/VirtualizedTree/VirtualizedTreeRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** Icons the tree draws in the chevron column, chosen per row by what the row is. */
export interface IVirtualizedTreeIcons {
  /** A directory that is closed. */
  collapsed: ReactNode;
  /** A directory that is open. */
  expanded: ReactNode;
  /** A leaf. */
  leaf: ReactNode;
}

export interface IVirtualizedTreeProps<T> extends BaseComponentProps {
  items: Array<IPathTreeItem<T>>;
  expandedIds: ReadonlySet<string>;
  selectedId: Nullable<string>;
  ariaLabel: string;
  icons: IVirtualizedTreeIcons;
  /** Decorates a leaf's label, for a consumer that marks where the entry came from. */
  renderLabel?: (item: IPathTreeItem<T>) => ReactNode;
  onToggleExpanded: (id: string) => void;
  /**
   * A row was chosen, directory or leaf.
   */
  onSelect: (item: IPathTreeItem<T>) => void;
}

/**
 * A path tree that renders only the rows on screen.
 *
 * Cost is the size of the viewport rather than the size of the directory, which is the whole point: MUI's
 * `RichTreeView` renders every revealed row, and a directory of two thousand entries costs a third of a
 * second to open. Virtualizing requires a flat DOM - a window of rows cannot be sliced out of nested
 * lists - so this owns the tree semantics that nesting used to provide.
 *
 * The keyboard position is tracked as an index and published with `aria-activedescendant` rather than by
 * focusing rows. Focus stays on the tree, so scrolling the active row out of the rendered window cannot
 * destroy it, which is the failure mode a roving `tabIndex` has once rows are virtualized.
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
}: IVirtualizedTreeProps<T>): ReactElement {
  const treeId: string = useId();
  const [activeIndex, setActiveIndex] = useState<number>(0);

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

  const activate = useCallback(
    (row: IFlatTreeRow<T>) => {
      // A directory both opens and reports, matching what clicking a row on the content did before: an
      // empty one has nothing to open but is still selectable.
      if (row.hasChildren) {
        onToggleExpanded(row.item.id);
      }

      onSelect(row.item);
    },
    [onSelect, onToggleExpanded]
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
      const icon: ReactNode = row.hasChildren ? (row.isExpanded ? icons.expanded : icons.collapsed) : icons.leaf;

      return (
        <VirtualizedTreeRow<T>
          key={row.item.id}
          row={row}
          rowId={rowIdOf(params.rowIndex)}
          icon={icon}
          isActive={params.rowIndex === activeIndex}
          isSelected={row.item.id === selectedId}
          label={row.item.kind === "file" ? renderLabel?.(row.item) : undefined}
          onActivate={(it: IFlatTreeRow<T>) => {
            setActiveIndex(params.rowIndex);
            activate(it);
          }}
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

  // Rows come and go as directories open, so a remembered position can outlive the list it indexed.
  useEffect(() => {
    setActiveIndex((current: number) => Math.min(Math.max(current, 0), Math.max(rows.length - 1, 0)));
  }, [rows.length]);

  /** Scrolls a row into view by arithmetic, since every row is exactly one `TREE.rowHeight` tall. */
  const revealRow = useCallback((index: number): void => {
    const scroller: Nullable<HTMLElement> = scrollerRef.current;

    if (!scroller) {
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

      setActiveIndex(next);
      revealRow(next);
    },
    [revealRow, rows.length]
  );

  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>): void => {
      const row: Nullable<IFlatTreeRow<T>> = rows[activeIndex] ?? null;

      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();

          return moveTo(activeIndex + 1);

        case "ArrowUp":
          event.preventDefault();

          return moveTo(activeIndex - 1);

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
            return moveTo(activeIndex + 1);
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

        case "Enter":
        case " ": {
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
    [activate, activeIndex, moveTo, onToggleExpanded, rows]
  );

  return (
    <Box
      {...containerProps}
      ref={setScroller}
      aria-activedescendant={rows.length ? rowIdOf(activeIndex) : undefined}
      aria-label={ariaLabel}
      data-testid={dataTestId}
      id={id}
      className={className}
      role={"tree"}
      sx={{ height: "100%", outline: "none", overflow: "auto", padding: 0.5, ...sx }}
      tabIndex={0}
      onKeyDown={onKeyDown}
    >
      <div {...contentProps} />
      <div {...positionerProps} role={"presentation"} />

      {virtualizer.api.getters.getRows()}
    </Box>
  );
}
