import { Box, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * Where an open document actually is on disk.
 */
export interface IEditorLocation {
  /** The file on disk: the document itself, or the archive volume holding it. */
  path: string;
  /** The entry's own name inside that volume, absent for a file that is simply on disk. */
  entry?: Nullable<string>;
}

interface IEditorToolbarLocationProps extends BaseComponentProps {
  location: IEditorLocation;
}

/**
 * Where the open document is, on the toolbar's last crumb.
 *
 * Truncated from the left by the toolbar, which is what keeps the end of a path - the part that identifies the file -
 * on screen when the beginning does not fit. The tooltip carries both addresses in full, since the whole reason to
 * show a location is the times somebody needs to copy or check it.
 */
export function EditorToolbarLocation({
  "data-testid": dataTestId = "editor-toolbar-location",
  id,
  className,
  location,
}: IEditorToolbarLocationProps): ReactElement {
  const { path, entry } = location;

  return (
    <Tooltip title={entry ? `${path}\n${entry}` : path} slotProps={{ tooltip: { sx: { whiteSpace: "pre-line" } } }}>
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        component={"span"}
        sx={{ fontFamily: MONOSPACE.fontFamily }}
      >
        {path}
        {entry ? (
          <>
            <Box component={"span"} aria-hidden={true} sx={{ opacity: 0.5, paddingX: 0.5 }}>
              &rsaquo;
            </Box>
            {entry}
          </>
        ) : null}
      </Box>
    </Tooltip>
  );
}
