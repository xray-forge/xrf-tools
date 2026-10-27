import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

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
    <Typography data-testid={dataTestId} id={id} className={cn("monospace break-all", className)} variant={"body2"}>
      {value}
    </Typography>
  );
}
