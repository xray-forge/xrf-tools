import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ARCHIVE_PATH_TEXT } from "@/applications/archives-explorer/components/editor/archive-editor.styles";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchiveFileDetailRowProps extends BaseComponentProps {
  isPath?: boolean;
  label: string;
  value: string;
  mono?: boolean;
}

export function ArchiveFileDetailRow({
  isPath = false,
  label,
  value,
  mono = false,
}: IArchiveFileDetailRowProps): ReactElement {
  return (
    <Box sx={{ marginBottom: 1.5 }}>
      <Typography variant={"caption"} sx={{ display: "block", color: "text.secondary" }}>
        {label}
      </Typography>

      <Typography variant={"body2"} sx={isPath || mono ? ARCHIVE_PATH_TEXT : { overflowWrap: "anywhere" }}>
        {value}
      </Typography>
    </Box>
  );
}
