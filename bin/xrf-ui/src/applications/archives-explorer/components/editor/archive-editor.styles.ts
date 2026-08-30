import { SxProps, Theme } from "@mui/material";

import { MONOSPACE } from "@/core/theme/tokens";

/** Entry names and volume paths are compared by eye, so they never render in proportional type. */
export const ARCHIVE_EDITOR_MONOSPACE_FONT: string = MONOSPACE.fontFamily;

/**
 * One long identifier on its own line: an entry name, a volume path, an engine path.
 *
 * Wrapping anywhere rather than on word boundaries, because a backslash-separated path has none a panel this narrow can
 * use.
 */
export const ARCHIVE_PATH_TEXT: SxProps<Theme> = {
  ...MONOSPACE,
  overflowWrap: "anywhere",
};
