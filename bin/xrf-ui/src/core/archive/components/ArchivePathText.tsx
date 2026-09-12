import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

interface IArchivePathTextProps extends BaseComponentProps {
  value: string;
}

/**
 * A filesystem path, shown in full.
 *
 * Paths are long and their tail is the part that identifies them, so they wrap rather than truncate.
 */
export function ArchivePathText({
  "data-testid": dataTestId = "archive-path-text",
  id,
  className,
  value,
}: IArchivePathTextProps): ReactElement {
  return (
    <Typography
      data-testid={dataTestId}
      id={id}
      className={className ? `monospace ${className}` : "monospace"}
      variant={"body2"}
      sx={{ wordBreak: "break-all" }}
    >
      {value}
    </Typography>
  );
}
