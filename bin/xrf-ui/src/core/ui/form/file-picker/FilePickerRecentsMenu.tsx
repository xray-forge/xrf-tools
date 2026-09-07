import { default as CheckIcon } from "@mui/icons-material/Check";
import { default as CloseIcon } from "@mui/icons-material/Close";
import { Box, IconButton, ListItemText, Menu, MenuItem, Tooltip, Typography } from "@mui/material";
import { formatDistanceToNowStrict } from "date-fns";
import { MouseEvent, ReactElement } from "react";

import { MONOSPACE, MONOSPACE_CHARACTER_WIDTH } from "@/core/theme/tokens";
import { IPathFieldRecents, IPathRecord } from "@/core/ui/form/path-recents";
import { isSamePath, truncatePathHead } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

/** Room a row spends on things that are not the path: the mark, the age, the control that forgets it, the padding. */
const ROW_FURNITURE_WIDTH: number = 190;

/** Below this there is no point shortening further, so a path is cut mid-name instead. */
const MINIMUM_LABEL_LIMIT: number = 16;

/** Used until the field has been measured, and when it reports nothing. */
const FALLBACK_MENU_WIDTH: number = 560;

/** Width of the column holding the mark, kept whether or not a row carries one. */
const MARK_COLUMN_WIDTH: number = 20;

/**
 * How many characters of a path one row can show.
 *
 * Derived rather than fixed because the same control is a few hundred pixels wide in a side panel and over a thousand
 * in a picker form, and a budget chosen for one of those cuts paths that had room in the other.
 *
 * @param width - Width of the menu, in pixels.
 * @returns The most characters a row may show.
 */
function resolveLabelLimit(width: number): number {
  return Math.max(MINIMUM_LABEL_LIMIT, Math.floor((width - ROW_FURNITURE_WIDTH) / MONOSPACE_CHARACTER_WIDTH));
}

interface IFilePickerRecentsMenuProps {
  isOpen: boolean;
  /**
   * The field itself, not the control that opens this.
   */
  anchor: Nullable<HTMLElement>;
  recents: IPathFieldRecents;
  /** Marked rather than hidden, so the list does not shift between openings. */
  currentPath?: Nullable<string>;
  onClose: () => void;
}

/**
 * The paths this field was given before.
 */
export function FilePickerRecentsMenu({
  isOpen,
  anchor,
  recents,
  currentPath = null,
  onClose,
}: IFilePickerRecentsMenuProps): ReactElement {
  const width: number = anchor?.offsetWidth || FALLBACK_MENU_WIDTH;

  return (
    <Menu
      open={Boolean(isOpen && anchor)}
      anchorEl={anchor}
      anchorOrigin={{ horizontal: "left", vertical: "bottom" }}
      transformOrigin={{ horizontal: "left", vertical: "top" }}
      slotProps={{ paper: { sx: { maxWidth: "100%", width } } }}
      onClose={onClose}
    >
      {recents.records.map((it: IPathRecord) => {
        const isCurrent: boolean = currentPath !== null && isSamePath(it.path, currentPath);

        return (
          <MenuItem
            key={it.path}
            selected={isCurrent}
            sx={{ gap: 1 }}
            onClick={() => {
              recents.pick(it.path);
              onClose();
            }}
          >
            <Box sx={{ display: "flex", flexShrink: 0, justifyContent: "center", width: MARK_COLUMN_WIDTH }}>
              {isCurrent ? <CheckIcon fontSize={"small"} color={"primary"} /> : null}
            </Box>

            <Tooltip describeChild placement={"top"} title={it.path}>
              <ListItemText
                primary={truncatePathHead(it.path, resolveLabelLimit(width))}
                slotProps={{ primary: { sx: { ...MONOSPACE, overflow: "hidden", whiteSpace: "nowrap" } } }}
                sx={{ minWidth: 0 }}
              />
            </Tooltip>

            <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
              {formatDistanceToNowStrict(it.at, { addSuffix: true })}
            </Typography>

            <Tooltip describeChild title={"Forget"}>
              <IconButton
                aria-label={`Forget ${it.path}`}
                size={"small"}
                sx={{ flexShrink: 0 }}
                // Stops the row underneath from also being picked, which would fill the field with what was forgotten.
                onClick={(event: MouseEvent<HTMLElement>) => {
                  event.stopPropagation();
                  recents.forget(it.path);
                }}
              >
                <CloseIcon fontSize={"inherit"} />
              </IconButton>
            </Tooltip>
          </MenuItem>
        );
      })}
    </Menu>
  );
}
