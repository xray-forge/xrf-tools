import { Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveDescribeScope, ArchiveLevelWallmarkSlot } from "@/core/ipc/types/xrf-app";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeReferenceStatus, formatCount } from "../ArchiveDescriptionPreview.utils";
import { ArchiveDescriptionReferenceLink } from "../ArchiveDescriptionReferenceLink";

interface IArchiveLevelWallmarkSlotRowProps extends BaseComponentProps {
  slot: ArchiveLevelWallmarkSlot;
  scope: ArchiveDescribeScope;
}

/**
 * One material of the baked decals: the blender and the texture they draw with, and what they cost to draw.
 */
export function ArchiveLevelWallmarkSlotRow({
  "data-testid": dataTestId = "archive-level-wallmark-slot-row",
  id,
  className,
  slot,
  scope,
}: IArchiveLevelWallmarkSlotRowProps): ReactElement {
  const status: Nullable<string> = slot.texture ? describeReferenceStatus(slot.texture, scope) : null;

  return (
    <div data-testid={dataTestId} id={id} className={cn("min-w-0 py-1.5 leading-panel", className)}>
      <div className={"flex min-w-0 justify-between gap-2"}>
        <Typography className={"monospace min-w-0 wrap-anywhere"} variant={"body2"}>
          {slot.texture ? <ArchiveDescriptionReferenceLink reference={slot.texture} /> : "Names no texture"}
        </Typography>

        <Typography className={"shrink-0 whitespace-nowrap text-text-secondary"} variant={"body2"}>
          {`${formatCount(slot.marks)} ${slot.marks === 1 ? "decal" : "decals"}`}
        </Typography>
      </div>

      <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
        {`${slot.shader} · ${formatCount(slot.vertices)} vertices`}
      </Typography>

      {status ? (
        <Typography className={"block wrap-anywhere text-text-disabled"} variant={"caption"}>
          {status}
        </Typography>
      ) : null}
    </div>
  );
}
