import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { XrayPathCollision } from "@/core/ipc/types/xrf-vfs";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ARCHIVE_OVERRIDE_ROW_HEIGHT } from "./archive-override-rows";
import { ArchiveUnreachableSiteRow } from "./ArchiveUnreachableSiteRow";

export interface IArchiveUnreachableRowProps extends BaseComponentProps {
  collision: XrayPathCollision;
}

/**
 * One engine identity two entries of a source claim, standing over both of them.
 */
export function ArchiveUnreachableRow({
  "data-testid": dataTestId = "archive-unreachable-row",
  id,
  className,
  collision,
}: IArchiveUnreachableRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <div className={cn("flex items-center gap-2 px-2")} style={{ height: ARCHIVE_OVERRIDE_ROW_HEIGHT }}>
        <Typography className={"monospace min-w-0 grow overflow-hidden text-ellipsis"} variant={"body2"} noWrap={true}>
          {collision.logicalPath}
        </Typography>

        <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
          2 copies
        </Typography>
      </div>

      <ArchiveUnreachableSiteRow site={collision.kept} rank={1} isKept />

      <ArchiveUnreachableSiteRow site={collision.unreachable} rank={2} isKept={false} />
    </div>
  );
}
