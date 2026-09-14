import { Box, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ArchiveResolutionVolume } from "@/core/ipc/types/xrf-app";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

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
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", alignItems: "baseline", gap: 1, paddingY: 0.25 }}
    >
      <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, width: 20 }}>
        {rank}
      </Typography>

      <Typography variant={"caption"} sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0, overflowWrap: "anywhere" }}>
        {volume.path}
      </Typography>

      <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
        {`${volume.entries.toLocaleString()} entries · ${formatBytes(volume.sizeReal)}`}
      </Typography>
    </Box>
  );
}
