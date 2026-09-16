import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveUnreadSource } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
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
    <div data-testid={dataTestId} id={id} className={cn("border-t border-divider py-2", className)}>
      <Typography className={"monospace wrap-anywhere"} variant={"body2"}>
        {source.origin}
      </Typography>

      <Typography className={"monospace wrap-anywhere text-text-secondary"} variant={"body2"}>
        {source.path}
      </Typography>

      <Typography className={"block wrap-anywhere text-warning"} variant={"caption"}>
        {source.reason}
      </Typography>
    </div>
  );
}
