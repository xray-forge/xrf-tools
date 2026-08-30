import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ARCHIVE_PATH_TEXT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { XrayPathCollision } from "@/core/bindings/types/xrf-vfs";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveCollisionRowProps extends BaseComponentProps {
  collision: XrayPathCollision;
}

/**
 * One engine path and the two files claiming it.
 *
 * Both sites are shown as authored, because the authored spelling is exactly what the fold destroys and the only thing
 * that says which of the two to remove.
 */
export function ArchiveCollisionRow({ collision }: IArchiveCollisionRowProps): ReactElement {
  return (
    <Box sx={{ padding: 2, paddingBottom: 1.5 }}>
      <Typography variant={"body2"} sx={ARCHIVE_PATH_TEXT}>
        {collision.logicalPath}
      </Typography>

      <Typography variant={"caption"} sx={{ display: "block", marginTop: 1, color: "text.secondary" }}>
        Unreachable
      </Typography>
      <Typography variant={"body2"} sx={{ ...ARCHIVE_PATH_TEXT, color: "warning.main" }}>
        {collision.unreachable}
      </Typography>

      <Typography variant={"caption"} sx={{ display: "block", marginTop: 1, color: "text.secondary" }}>
        Answers instead
      </Typography>
      <Typography variant={"body2"} sx={ARCHIVE_PATH_TEXT}>
        {collision.kept}
      </Typography>
    </Box>
  );
}
