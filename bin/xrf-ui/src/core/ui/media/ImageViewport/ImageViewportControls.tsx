import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as FitScreenIcon } from "@mui/icons-material/FitScreen";
import { default as ZoomInIcon } from "@mui/icons-material/ZoomIn";
import { default as ZoomOutIcon } from "@mui/icons-material/ZoomOut";
import { IconButton, Paper, Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

interface IImageViewportControlsProps {
  /** Current magnification, as the viewport resolved it.  */
  scale: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onActualSize: () => void;
  onFit: () => void;
}

/**
 * The zoom bar in the corner of a viewport.
 */
export function ImageViewportControls({
  scale,
  onZoomIn,
  onZoomOut,
  onActualSize,
  onFit,
}: IImageViewportControlsProps): ReactElement {
  return (
    <Paper
      variant={"outlined"}
      sx={{
        position: "absolute",
        right: 8,
        bottom: 8,
        display: "flex",
        alignItems: "center",
        gap: 0.5,
        padding: 0.5,
      }}
    >
      <Tooltip describeChild title={"Zoom out"}>
        <IconButton aria-label={"Zoom out"} size={"small"} onClick={onZoomOut}>
          <ZoomOutIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>

      <Typography variant={"caption"} sx={{ minWidth: 44, textAlign: "center", color: "text.secondary" }}>
        {Math.round(scale * 100)}%
      </Typography>

      <Tooltip describeChild title={"Zoom in"}>
        <IconButton aria-label={"Zoom in"} size={"small"} onClick={onZoomIn}>
          <ZoomInIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>

      <Tooltip describeChild title={"Actual size"}>
        <IconButton aria-label={"Actual size"} size={"small"} onClick={onActualSize}>
          <CenterFocusStrongIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>

      <Tooltip describeChild title={"Fit to view"}>
        <IconButton aria-label={"Fit to view"} size={"small"} onClick={onFit}>
          <FitScreenIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>
    </Paper>
  );
}
