import { Box } from "@mui/material";
import { ReactElement, ReactNode } from "react";

import { TREE } from "@/core/theme/tokens";
import { IFlatTreeRow } from "@/core/ui/tree/flatten";
import { IPathTreeItem } from "@/core/ui/tree/path-tree";

export interface IVirtualizedTreeRowProps<T> {
  row: IFlatTreeRow<T>;
  /** Element id, so the tree can point `aria-activedescendant` at the focused row. */
  rowId: string;
  isSelected: boolean;
  isActive: boolean;
  /** Drawn in the chevron column: open, closed, or leaf, chosen by the tree. */
  icon: ReactNode;
  /** Overrides the plain label, for a consumer that decorates its leaves. */
  label?: ReactNode;
  onActivate: (row: IFlatTreeRow<T>) => void;
}

/**
 * One row of a virtualized tree.
 *
 * A sibling of every other row rather than a nested list item, because a window of rows is what can be
 * virtualized at all. Everything nesting used to say is stated instead: `aria-level` for the depth the
 * indentation shows, and `aria-setsize` with `aria-posinset` for the position among siblings that a
 * flat DOM no longer implies.
 */
export function VirtualizedTreeRow<T>({
  row,
  rowId,
  isSelected,
  isActive,
  icon,
  label,
  onActivate,
}: IVirtualizedTreeRowProps<T>): ReactElement {
  const item: IPathTreeItem<T> = row.item;

  return (
    <Box
      aria-expanded={row.hasChildren ? row.isExpanded : undefined}
      aria-level={row.depth + 1}
      aria-posinset={row.posInSet}
      aria-selected={isSelected}
      aria-setsize={row.setSize}
      data-active={isActive ? "" : undefined}
      data-testid={"virtualized-tree-row"}
      id={rowId}
      role={"treeitem"}
      sx={{
        alignItems: "center",
        backgroundColor: isSelected ? "action.selected" : "transparent",
        borderRadius: 1,
        boxSizing: "border-box",
        cursor: "pointer",
        display: "flex",
        gap: `${TREE.iconGap}px`,
        height: TREE.rowHeight,
        // Indentation replaces the nesting a flat DOM gave up, so it has to be paid per level here.
        paddingLeft: `${row.depth * TREE.indent + 4}px`,
        paddingRight: 0.5,
        userSelect: "none",
        "&:hover": { backgroundColor: isSelected ? "action.selected" : "action.hover" },
        // The keyboard's position is drawn rather than focused: focus stays on the tree so that
        // scrolling a row out of the rendered window cannot destroy it.
        "&[data-active]": { outline: "1px solid", outlineColor: "primary.main", outlineOffset: "-1px" },
      }}
      onClick={() => onActivate(row)}
    >
      <Box
        sx={{
          alignItems: "center",
          color: "text.secondary",
          display: "flex",
          flexShrink: 0,
          justifyContent: "center",
          width: TREE.iconWidth,
          "& svg": { fontSize: TREE.iconSize },
        }}
      >
        {icon}
      </Box>

      <Box
        component={"span"}
        sx={{
          minWidth: 0,
          overflow: "hidden",
          textOverflow: "ellipsis",
          typography: "body2",
          whiteSpace: "nowrap",
        }}
      >
        {label ?? item.label}
      </Box>
    </Box>
  );
}
