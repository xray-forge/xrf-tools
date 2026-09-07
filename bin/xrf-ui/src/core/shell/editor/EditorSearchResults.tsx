import { Box, LinearProgress, List, ListItemButton, ListItemIcon, ListItemText, Typography } from "@mui/material";
import { ReactElement, ReactNode, useEffect, useRef } from "react";

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
      <Box sx={{ height: 2, flexShrink: 0 }}>
        <LinearProgress sx={{ height: 2 }} />
      </Box>
    ) : (
      <Box sx={{ padding: 2, textAlign: "center" }}>
        <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
          {emptyLabel}
        </Typography>
      </Box>
    );
  }

  return (
    <Box sx={{ display: "flex", flexDirection: "column", minHeight: 0 }}>
      {/* Only appears while the list belongs to an older query than the field. */}
      <Box sx={{ height: 2, flexShrink: 0 }}>{isStale ? <LinearProgress sx={{ height: 2 }} /> : null}</Box>

      {total > rows.length ? (
        <Typography variant={"caption"} sx={{ paddingX: 1.5, paddingY: 0.5, color: "text.secondary", flexShrink: 0 }}>
          Showing {rows.length} of {total} matches
        </Typography>
      ) : null}

      <List aria-label={ariaLabel} dense={true} disablePadding={true} sx={{ minHeight: 0, overflowY: "auto" }}>
        {rows.map((row: T, index: number) => (
          <ListItemButton
            key={row.id}
            ref={index === activeIndex ? activeRef : undefined}
            disabled={isDisabled}
            selected={index === activeIndex}
            sx={{ paddingY: 0.25 }}
            onMouseEnter={() => onHoverIndex(index)}
            onClick={() => onSelect(row)}
          >
            {row.icon ? <ListItemIcon sx={{ minWidth: 40 }}>{row.icon}</ListItemIcon> : null}

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
    </Box>
  );
}
