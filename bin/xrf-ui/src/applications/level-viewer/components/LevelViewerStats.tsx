import { Paper, Stack, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/level-stats";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelViewerStatsProps extends BaseComponentProps {
  stats: ILevelStats;
}

/**
 * What the viewport is costing, on screen from the first frame.
 */
export function LevelViewerStats({
  "data-testid": dataTestId = "level-viewer-stats",
  id,
  className,
  stats,
}: ILevelViewerStatsProps): ReactElement {
  return (
    <Paper
      data-testid={dataTestId}
      id={id}
      className={className}
      elevation={0}
      sx={{ bottom: 8, left: 8, opacity: 0.9, position: "absolute", px: 1.5, py: 0.75 }}
    >
      <Stack direction={"row"} spacing={2}>
        <Typography variant={"caption"}>{`${stats.framesPerSecond.toFixed(0)} fps`}</Typography>
        <Typography variant={"caption"}>{`${stats.frameTime.toFixed(1)} ms`}</Typography>
        <Typography variant={"caption"}>{`${stats.sectors} sectors`}</Typography>
        <Typography variant={"caption"}>{`${stats.draws} draws`}</Typography>
        <Typography variant={"caption"}>{`${(stats.triangles / 1000).toFixed(0)}k tris`}</Typography>
        <Typography variant={"caption"}>{`${(stats.bytes / 1024 / 1024).toFixed(1)} MB`}</Typography>
      </Stack>
    </Paper>
  );
}
