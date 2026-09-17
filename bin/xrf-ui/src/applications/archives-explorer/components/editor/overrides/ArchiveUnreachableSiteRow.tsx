import { Chip, chipClasses, Typography } from "@mui/material";
import { ReactElement } from "react";

import { BADGE_FONT_SIZE } from "@/core/theme/tokens";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { ARCHIVE_OVERRIDE_ROW_HEIGHT } from "./archive-override-rows";

export interface IArchiveUnreachableSiteRowProps extends BaseComponentProps {
  /** Where the copy physically sits, as the backend rendered it: a loose path, or a volume and the name it authored. */
  site: string;
  /** One-based place among the copies of this path, so the row reads as a rank rather than a position in a list. */
  rank: number;
  /** Whether this is the copy the source resolves, the other being the one nothing reaches. */
  isKept: boolean;
}

/**
 * One side of a collision, at its place among the copies claiming the path.
 */
export function ArchiveUnreachableSiteRow({
  "data-testid": dataTestId = "archive-unreachable-site-row",
  id,
  className,
  site,
  rank,
  isKept,
}: IArchiveUnreachableSiteRowProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex items-center gap-2 pr-2 pl-6", isKept ? "opacity-100" : "opacity-70", className)}
      style={{ height: ARCHIVE_OVERRIDE_ROW_HEIGHT }}
    >
      <Typography className={"w-4 shrink-0 text-text-secondary"} variant={"caption"}>
        {rank}
      </Typography>

      <Typography
        className={"monospace min-w-0 grow overflow-hidden text-ellipsis"}
        variant={"caption"}
        noWrap={true}
        title={site}
      >
        {site}
      </Typography>

      <Chip
        className={"h-4.5 shrink-0"}
        sx={{ [`& .${chipClasses.label}`]: { paddingX: 0.75, fontSize: BADGE_FONT_SIZE } }}
        size={"small"}
        color={isKept ? "primary" : "warning"}
        label={isKept ? "Loaded" : "Unreachable"}
        title={isKept ? "The copy the engine loads" : "No lookup reaches this copy"}
      />
    </div>
  );
}
