import { default as ChevronRightIcon } from "@mui/icons-material/ChevronRight";
import { default as ExpandMoreIcon } from "@mui/icons-material/ExpandMore";
import { CSSProperties, MouseEvent, ReactElement, ReactNode } from "react";

import { TREE } from "@/core/theme/tokens";
import { IFlatTreeRow } from "@/core/ui/tree/flatten";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ITreeIconDecoration } from "@/core/ui/tree/VirtualizedTree/VirtualizedTree";
import { Nullable } from "@/lib/types/general";

/** A theme colour path as the variable the theme emits for it, since a plain element has no `sx` to resolve it. */
function toPaletteColor(path: string): string {
  return `var(--mui-palette-${path.replaceAll(".", "-")})`;
}

interface IVirtualizedTreeRowProps<T> {
  row: IFlatTreeRow<T>;
  /** Element id, so the tree can point `aria-activedescendant` at the selected row. */
  rowId: string;
  /** Whether the keyboard stands here, which is where a selection moves and what a focus ring surrounds. */
  isSelected: boolean;
  /** Whether the editor has this row's subject open, which outlasts wherever the keyboard wandered since. */
  isActive: boolean;
  /** Drawn beside the chevron: open, closed, or leaf, chosen by the tree. Null for a tree that types nothing. */
  icon: Nullable<ReactNode>;
  /** How to tint that icon and what the tint says. Null leaves it in the neutral colour every other row uses. */
  iconDecoration: Nullable<ITreeIconDecoration>;
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
  isActive,
  icon,
  iconDecoration,
  label,
  onSelect,
  onActivate,
  onToggleExpanded,
}: IVirtualizedTreeRowProps<T>): ReactElement {
  const item: ITreeNode<T> = row.item;

  return (
    <div
      aria-current={isActive ? true : undefined}
      aria-expanded={row.hasChildren ? row.isExpanded : undefined}
      aria-level={row.depth + 1}
      aria-posinset={row.posInSet}
      aria-selected={isSelected}
      aria-setsize={row.setSize}
      data-active={isActive}
      data-selected={isSelected}
      data-testid={"virtualized-tree-row"}
      id={rowId}
      role={"treeitem"}
      className={
        "box-border flex h-tree-row cursor-pointer items-center gap-tree-gap rounded-surface pr-1 pl-[var(--row-indent)] select-none hover:bg-action-hover data-[active=true]:data-[selected=false]:bg-action-current data-[active=true]:data-[selected=false]:hover:bg-action-current data-[selected=true]:bg-action-selected data-[selected=true]:hover:bg-action-selected"
      }
      style={{ "--row-indent": `${row.depth * TREE.indent + 4}px` } as CSSProperties}
      onClick={() => onSelect(row)}
      onDoubleClick={() => onActivate(row)}
    >
      <div
        data-testid={"virtualized-tree-chevron"}
        className={"flex w-tree-icon shrink-0 items-center justify-center text-text-secondary [&_svg]:text-tree-icon"}
        onClick={(event: MouseEvent<HTMLElement>) => {
          event.stopPropagation();

          if (row.hasChildren) {
            onToggleExpanded(item.id);
          }
        }}
        onDoubleClick={(event: MouseEvent<HTMLElement>) => event.stopPropagation()}
      >
        {row.hasChildren ? row.isExpanded ? <ExpandMoreIcon /> : <ChevronRightIcon /> : null}
      </div>

      {icon === null ? null : (
        <div
          title={iconDecoration?.title}
          className={"flex w-tree-icon shrink-0 items-center justify-center text-text-secondary [&_svg]:text-tree-icon"}
          style={iconDecoration ? { color: toPaletteColor(iconDecoration.color) } : undefined}
        >
          {icon}
        </div>
      )}

      <span
        className={
          "min-w-0 overflow-hidden tracking-body2 text-ellipsis whitespace-nowrap [font:var(--mui-font-body2)] data-[active=true]:font-medium"
        }
        data-active={isActive}
      >
        {label ?? item.label}
      </span>
    </div>
  );
}
