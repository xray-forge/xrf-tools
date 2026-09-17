import { LinearProgress, List, ListItemButton, ListItemIcon, ListItemText, Typography } from "@mui/material";
import { ReactElement, ReactNode, useEffect, useRef } from "react";

import { EmptyListing } from "@/core/ui/layout/EmptyListing";

export interface IEditorSearchResultRow {
  id: string;
  label: string;
  /** Shown muted beneath the label, for the directory a match came from. */
  description?: string;
  icon?: ReactNode;
}

export interface IEditorSearchResultsProps<T extends IEditorSearchResultRow> {
  ariaLabel: string;
  emptyLabel: string;
  rows: Array<T>;
  /** Matches found, which exceeds `rows.length` once the limit applies. */
  total: number;
  activeIndex: number;
  isStale?: boolean;
  /** Set while a read or write is in flight, so a second selection cannot outrun the first. */
  isDisabled?: boolean;
  onHoverIndex: (index: number) => void;
  onSelect: (row: T) => void;
}

/**
 * Flat result list for filter-as-you-type panels.
 */
export function EditorSearchResults<T extends IEditorSearchResultRow>({
  ariaLabel,
  emptyLabel,
  rows,
  total,
  activeIndex,
  isStale,
  isDisabled,
  onHoverIndex,
  onSelect,
}: IEditorSearchResultsProps<T>): ReactElement {
  const activeRef = useRef<HTMLDivElement>(null);

  // Keyboard selection is useless if the row it lands on is below the fold.
  useEffect(() => {
    activeRef.current?.scrollIntoView({ block: "nearest" });
  }, [activeIndex, rows]);

  if (!rows.length) {
    // While stale the list belongs to an older query, so an empty one means "not filtered yet", not
    // "nothing matches". Claiming the latter makes every first keystroke flash a false negative.
    return isStale ? (
      <div className={"h-0.5 shrink-0"}>
        <LinearProgress className={"h-0.5"} />
      </div>
    ) : (
      <EmptyListing label={emptyLabel} />
    );
  }

  return (
    <div className={"flex min-h-0 flex-col"}>
      {/* Only appears while the list belongs to an older query than the field. */}
      <div className={"h-0.5 shrink-0"}>{isStale ? <LinearProgress className={"h-0.5"} /> : null}</div>

      {total > rows.length ? (
        <Typography className={"shrink-0 px-3 py-1 text-text-secondary"} variant={"caption"}>
          Showing {rows.length} of {total} matches
        </Typography>
      ) : null}

      <List aria-label={ariaLabel} className={"min-h-0 overflow-y-auto"} dense={true} disablePadding={true}>
        {rows.map((row: T, index: number) => (
          <ListItemButton
            key={row.id}
            ref={index === activeIndex ? activeRef : undefined}
            className={"py-0.5"}
            disabled={isDisabled}
            selected={index === activeIndex}
            onMouseEnter={() => onHoverIndex(index)}
            onClick={() => onSelect(row)}
          >
            {row.icon ? <ListItemIcon className={"min-w-10"}>{row.icon}</ListItemIcon> : null}

            <ListItemText
              primary={row.label}
              secondary={row.description}
              slotProps={{
                primary: { variant: "body2", noWrap: true, title: row.label },
                secondary: { variant: "caption", noWrap: true },
              }}
            />
          </ListItemButton>
        ))}
      </List>
    </div>
  );
}
