import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as ZoomInIcon } from "@mui/icons-material/ZoomIn";
import { default as ZoomOutIcon } from "@mui/icons-material/ZoomOut";
import { ButtonBase, IconButton, Paper, Tooltip, Typography } from "@mui/material";
import { ReactElement } from "react";

interface IViewportControlsProps {
  /**
   * Current magnification and how to return to one to one, for a viewport where that means something.
   */
  zoom?: {
    scale: number;
    onActualSize: () => void;
  };
  onZoomIn: () => void;
  onZoomOut: () => void;
  onReset: () => void;
}

/**
 * The camera bar in the corner of a viewport, shared by the picture and by both scenes.
 */
export function ViewportControls({ zoom, onZoomIn, onZoomOut, onReset }: IViewportControlsProps): ReactElement {
  return (
    <Paper
      data-testid={"viewport-controls"}
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

      {zoom ? (
        <Tooltip describeChild title={"Actual size"}>
          <ButtonBase
            aria-label={"Actual size"}
            sx={{ borderRadius: 1, paddingX: 0.5, paddingY: 0.25 }}
            onClick={zoom.onActualSize}
          >
            <Typography variant={"caption"} sx={{ minWidth: 44, textAlign: "center", color: "text.secondary" }}>
              {Math.round(zoom.scale * 100)}%
            </Typography>
          </ButtonBase>
        </Tooltip>
      ) : null}

      <Tooltip describeChild title={"Zoom in"}>
        <IconButton aria-label={"Zoom in"} size={"small"} onClick={onZoomIn}>
          <ZoomInIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>

      <Tooltip describeChild title={"Reset view"}>
        <IconButton aria-label={"Reset view"} size={"small"} onClick={onReset}>
          <CenterFocusStrongIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>
    </Paper>
  );
}
