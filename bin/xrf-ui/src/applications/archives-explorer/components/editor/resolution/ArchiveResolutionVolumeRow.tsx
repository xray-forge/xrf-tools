import { Typography } from "@mui/material";
import { formatBytes, formatCount } from "@xrf/format";
import { ReactElement } from "react";

import { ArchiveResolutionVolume } from "@/core/ipc/types/xrf-app-archives";
import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IArchiveResolutionVolumeRowProps extends BaseComponentProps {
  volume: ArchiveResolutionVolume;
  /** Position of the volume in the set's own search order, counted from one. */
  rank: number;
}

/**
 * One volume of a set, at the position a lookup reaches it.
 */
export function ArchiveResolutionVolumeRow({
  "data-testid": dataTestId = "archive-resolution-volume-row",
  id,
  className,
  volume,
  rank,
}: IArchiveResolutionVolumeRowProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex items-baseline gap-2 py-0.5", className)}>
      <Typography className={"w-5 shrink-0 text-text-secondary"} variant={"caption"}>
        {rank}
      </Typography>

      <Typography className={"monospace min-w-0 grow wrap-anywhere"} variant={"caption"}>
        {volume.path}
      </Typography>

      <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
        {`${formatCount(volume.entries)} entries · ${formatBytes(volume.sizeReal)}`}
      </Typography>
    </div>
  );
}
