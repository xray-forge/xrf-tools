import { formatPercent } from "@xrf/format";
import { CenterFocusStrongIcon, ZoomInIcon, ZoomOutIcon } from "@xrf/ui/icons";
import { ButtonBase, IconButton, Paper, Tooltip, Typography } from "@xrf/ui/material";
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
      className={"absolute right-2 bottom-2 flex items-center gap-1 p-1"}
      variant={"outlined"}
    >
      <Tooltip describeChild title={"Zoom out"}>
        <IconButton aria-label={"Zoom out"} size={"small"} onClick={onZoomOut}>
          <ZoomOutIcon fontSize={"small"} />
        </IconButton>
      </Tooltip>

      {zoom ? (
        <Tooltip describeChild title={"Actual size"}>
          <ButtonBase aria-label={"Actual size"} className={"rounded-surface px-1 py-0.5"} onClick={zoom.onActualSize}>
            <Typography className={"min-w-11 text-center text-text-secondary"} variant={"caption"}>
              {formatPercent(zoom.scale)}
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
