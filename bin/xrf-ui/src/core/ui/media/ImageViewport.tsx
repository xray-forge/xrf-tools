import { default as CenterFocusStrongIcon } from "@mui/icons-material/CenterFocusStrong";
import { default as FitScreenIcon } from "@mui/icons-material/FitScreen";
import { default as ZoomInIcon } from "@mui/icons-material/ZoomIn";
import { default as ZoomOutIcon } from "@mui/icons-material/ZoomOut";
import { Box, IconButton, Paper, Tooltip, Typography } from "@mui/material";
import { MouseEvent, ReactElement, useCallback, useEffect, useRef, useState, WheelEvent } from "react";

import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import {
  clampScale,
  fitToViewport,
  IPanZoomState,
  PAN_ZOOM_IDENTITY,
  panBy,
  zoomAround,
  zoomByWheel,
} from "@/lib/media/pan-zoom";
import { Nullable } from "@/lib/types/general";

interface IImageViewportProps {
  src: string;
  alt: string;
  width: number;
  height: number;
}

/**
 * Pannable, zoomable viewport for a single image.
 */
export function ImageViewport({ src, alt, width, height }: IImageViewportProps): ReactElement {
  const viewportRef = useRef<Nullable<HTMLDivElement>>(null);
  const dragOriginRef = useRef<Nullable<{ x: number; y: number }>>(null);

  const [state, setState] = useState<IPanZoomState>(PAN_ZOOM_IDENTITY);

  const getViewportSize = useCallback((): { x: number; y: number } => {
    const element: Nullable<HTMLDivElement> = viewportRef.current;

    return { x: element?.clientWidth ?? 0, y: element?.clientHeight ?? 0 };
  }, []);

  const onFit = useCallback(() => {
    setState(fitToViewport({ x: width, y: height }, getViewportSize()));
  }, [getViewportSize, height, width]);

  const onWheel = useCallback((event: WheelEvent<HTMLDivElement>) => {
    const bounds: Nullable<DOMRect> = viewportRef.current?.getBoundingClientRect() ?? null;

    if (!bounds) {
      return;
    }

    const point = { x: event.clientX - bounds.left, y: event.clientY - bounds.top };

    setState((current: IPanZoomState) => zoomByWheel(current, point, event.deltaY));
  }, []);

  const onZoomStep = useCallback(
    (factor: number) => {
      const size = getViewportSize();

      setState((current: IPanZoomState) =>
        zoomAround(current, { x: size.x / 2, y: size.y / 2 }, current.scale * factor)
      );
    },
    [getViewportSize]
  );

  const onActualSize = useCallback(() => {
    const size = getViewportSize();

    setState((current: IPanZoomState) => zoomAround(current, { x: size.x / 2, y: size.y / 2 }, 1));
  }, [getViewportSize]);

  const onMouseDown = useCallback((event: MouseEvent<HTMLDivElement>) => {
    dragOriginRef.current = { x: event.clientX, y: event.clientY };
  }, []);

  const onMouseMove = useCallback((event: MouseEvent<HTMLDivElement>) => {
    const origin: Nullable<{ x: number; y: number }> = dragOriginRef.current;

    if (!origin) {
      return;
    }

    const deltaX: number = event.clientX - origin.x;
    const deltaY: number = event.clientY - origin.y;

    dragOriginRef.current = { x: event.clientX, y: event.clientY };

    setState((current: IPanZoomState) => panBy(current, deltaX, deltaY));
  }, []);

  const onRelease = useCallback(() => {
    dragOriginRef.current = null;
  }, []);

  // A new image starts fitted rather than inheriting the previous one's pan, which would open it
  // somewhere off screen.
  useEffect(() => {
    setState(fitToViewport({ x: width, y: height }, getViewportSize()));
  }, [getViewportSize, height, src, width]);

  return (
    <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}>
      <Box
        ref={viewportRef}
        sx={[
          {
            position: "relative",
            flexGrow: 1,
            minHeight: 0,
            overflow: "hidden",
            cursor: "grab",
            "&:active": { cursor: "grabbing" },
            backgroundColor: "#353535",
          },
          IMAGE_CHECKERBOARD,
        ]}
        onWheel={onWheel}
        onMouseDown={onMouseDown}
        onMouseMove={onMouseMove}
        onMouseUp={onRelease}
        onMouseLeave={onRelease}
      >
        <Box
          component={"img"}
          alt={alt}
          src={src}
          draggable={false}
          sx={{
            position: "absolute",
            left: 0,
            top: 0,
            transformOrigin: "0 0",
            transform: `translate(${state.offsetX}px, ${state.offsetY}px) scale(${state.scale})`,
            imageRendering: state.scale > 1 ? "pixelated" : "auto",
            userSelect: "none",
          }}
        />

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
            <IconButton aria-label={"Zoom out"} size={"small"} onClick={() => onZoomStep(1 / 1.2)}>
              <ZoomOutIcon fontSize={"small"} />
            </IconButton>
          </Tooltip>

          <Typography variant={"caption"} sx={{ minWidth: 44, textAlign: "center", color: "text.secondary" }}>
            {Math.round(clampScale(state.scale) * 100)}%
          </Typography>

          <Tooltip describeChild title={"Zoom in"}>
            <IconButton aria-label={"Zoom in"} size={"small"} onClick={() => onZoomStep(1.2)}>
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
      </Box>
    </Box>
  );
}
