import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveUnreadSource } from "@/core/ipc/types/xrf-app";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IArchiveResolutionUnreadRowProps extends BaseComponentProps {
  source: ArchiveUnreadSource;
}

/**
 * One source the open subject declared and could not open.
 */
export function ArchiveResolutionUnreadRow({
  "data-testid": dataTestId = "archive-resolution-unread-row",
  id,
  className,
  source,
}: IArchiveResolutionUnreadRowProps): ReactElement {
  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ paddingY: 1, borderTop: 1, borderColor: "divider" }}
    >
      <Typography variant={"body2"} sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>
        {source.origin}
      </Typography>

      <Typography variant={"body2"} sx={{ ...MONOSPACE, color: "text.secondary", overflowWrap: "anywhere" }}>
        {source.path}
      </Typography>

      <Typography variant={"caption"} sx={{ color: "warning.main", display: "block", overflowWrap: "anywhere" }}>
        {source.reason}
      </Typography>
    </Box>
  );
}
