import { default as ChevronRightIcon } from "@mui/icons-material/ChevronRight";
import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import { Box } from "@mui/material";
import { MouseEvent, ReactElement, ReactNode } from "react";

import { TREE } from "@/core/theme/tokens";
import { IFlatTreeRow } from "@/core/ui/tree/flatten";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { Nullable } from "@/lib/types/general";

export interface IVirtualizedTreeRowProps<T> {
  row: IFlatTreeRow<T>;
  /** Element id, so the tree can point `aria-activedescendant` at the selected row. */
  rowId: string;
  isSelected: boolean;
  /** Drawn beside the chevron: open, closed, or leaf, chosen by the tree. Null for a tree that types nothing. */
  icon: Nullable<ReactNode>;
  /** Overrides the plain label, for a consumer that decorates its rows. */
  label?: ReactNode;
  onSelect: (row: IFlatTreeRow<T>) => void;
  onActivate: (row: IFlatTreeRow<T>) => void;
  onToggleExpanded: (id: string) => void;
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
  icon,
  label,
  onSelect,
  onActivate,
  onToggleExpanded,
}: IVirtualizedTreeRowProps<T>): ReactElement {
  const item: ITreeNode<T> = row.item;

  return (
    <Box
      aria-expanded={row.hasChildren ? row.isExpanded : undefined}
      aria-level={row.depth + 1}
      aria-posinset={row.posInSet}
      aria-selected={isSelected}
      aria-setsize={row.setSize}
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
      }}
      onClick={() => onSelect(row)}
      onDoubleClick={() => onActivate(row)}
    >
      <Box
        data-testid={"virtualized-tree-chevron"}
        sx={{
          alignItems: "center",
          color: "text.secondary",
          display: "flex",
          flexShrink: 0,
          justifyContent: "center",
          width: TREE.iconWidth,
          "& svg": { fontSize: TREE.iconSize },
        }}
        // Expansion is structural, so it leaves the selection where the user put it. The double click is
        // swallowed as well, or the row would activate and toggle a second time under the same gesture.
        onClick={(event: MouseEvent<HTMLElement>) => {
          event.stopPropagation();

          if (row.hasChildren) {
            onToggleExpanded(item.id);
          }
        }}
        onDoubleClick={(event: MouseEvent<HTMLElement>) => event.stopPropagation()}
      >
        {row.hasChildren ? row.isExpanded ? <ExpandMoreIcon /> : <ChevronRightIcon /> : null}
      </Box>

      {icon === null ? null : (
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
      )}

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
