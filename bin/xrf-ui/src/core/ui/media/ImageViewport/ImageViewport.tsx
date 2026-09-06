import { Box } from "@mui/material";
import {
  Dispatch,
  MouseEvent,
  ReactElement,
  RefCallback,
  SetStateAction,
  useCallback,
  useEffect,
  useRef,
  useState,
  WheelEvent,
} from "react";

import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import {
  IPanZoomCamera,
  IPanZoomPoint,
  IPanZoomSize,
  IPanZoomState,
  IPanZoomTransform,
  PAN_ZOOM_FIT,
  panBy,
  resolvePanZoomCamera,
  toContentPoint,
  toManualPanZoom,
  toPanZoomTransform,
  zoomAround,
  zoomByWheel,
} from "@/lib/media/pan-zoom";
import { IElementSize, useElementSize } from "@/lib/react";
import { Nullable } from "@/lib/types/general";

import { ImageViewportControls } from "./ImageViewportControls";

/** Stands in for the viewport until it has been measured, when there is nowhere to place anything anyway. */
const UNMEASURED: IPanZoomSize = { width: 0, height: 0 };

/** One wheel notch, as the direction the wheel reports it in. */
const ZOOM_IN: number = -1;
const ZOOM_OUT: number = 1;

interface IImageViewportProps {
  src: string;
  alt: string;
  width: number;
  height: number;
  /**
   * Where the camera looks, held by the caller, for two viewports that must show the same part of two pictures.
   */
  state?: IPanZoomState;
  onStateChange?: Dispatch<SetStateAction<IPanZoomState>>;
  /** Whether this viewport draws the zoom controls. Off for the second of a pair, which the first one moves. */
  hasControls?: boolean;
}

/**
 * Pannable, zoomable viewport for a single image.
 */
export function ImageViewport({
  src,
  alt,
  width,
  height,
  state: controlledState,
  hasControls = true,
  onStateChange,
}: IImageViewportProps): ReactElement {
  const [viewportRef, measured]: [RefCallback<HTMLDivElement>, Nullable<IElementSize>] =
    useElementSize<HTMLDivElement>();

  const [ownState, setOwnState] = useState<IPanZoomState>(PAN_ZOOM_FIT);
  const dragOriginRef = useRef<Nullable<IPanZoomPoint>>(null);

  const isControlled: boolean = controlledState !== undefined;
  const state: IPanZoomState = controlledState ?? ownState;
  const setState: Dispatch<SetStateAction<IPanZoomState>> = onStateChange ?? setOwnState;

  const transform: IPanZoomTransform = toPanZoomTransform(state, { width, height }, measured ?? UNMEASURED);

  /** Moves whichever camera the current state resolves to, and leaves the viewport looking through the result. */
  const moveCamera = useCallback(
    (move: (camera: IPanZoomCamera) => IPanZoomCamera): void => {
      setState((current: IPanZoomState) =>
        toManualPanZoom(move(resolvePanZoomCamera(current, { width, height }, measured ?? UNMEASURED)))
      );
    },
    [height, measured, setState, width]
  );

  const onWheel = useCallback(
    (event: WheelEvent<HTMLDivElement>): void => {
      // Measured here rather than read from the observer, because the rect is needed for the pointer's offset anyway
      // and answers both questions from one layout.
      const bounds: DOMRect = event.currentTarget.getBoundingClientRect();
      const point: IPanZoomPoint = { x: event.clientX - bounds.left, y: event.clientY - bounds.top };

      moveCamera((camera: IPanZoomCamera) => zoomByWheel(camera, toContentPoint(camera, bounds, point), event.deltaY));
    },
    [moveCamera]
  );

  // A button steps by exactly what a notch does, about the middle of what is on screen rather than the pointer.
  const onZoomIn = useCallback(
    (): void => moveCamera((camera: IPanZoomCamera) => zoomByWheel(camera, camera.center, ZOOM_IN)),
    [moveCamera]
  );

  const onZoomOut = useCallback(
    (): void => moveCamera((camera: IPanZoomCamera) => zoomByWheel(camera, camera.center, ZOOM_OUT)),
    [moveCamera]
  );

  const onActualSize = useCallback(
    (): void => moveCamera((camera: IPanZoomCamera) => zoomAround(camera, camera.center, 1)),
    [moveCamera]
  );

  const onFit = useCallback((): void => setState(PAN_ZOOM_FIT), [setState]);

  const onMouseDown = useCallback((event: MouseEvent<HTMLDivElement>): void => {
    dragOriginRef.current = { x: event.clientX, y: event.clientY };
  }, []);

  const onMouseMove = useCallback(
    (event: MouseEvent<HTMLDivElement>): void => {
      const origin: Nullable<IPanZoomPoint> = dragOriginRef.current;

      if (!origin) {
        return;
      }

      const deltaX: number = event.clientX - origin.x;
      const deltaY: number = event.clientY - origin.y;

      dragOriginRef.current = { x: event.clientX, y: event.clientY };

      moveCamera((camera: IPanZoomCamera) => panBy(camera, deltaX, deltaY));
    },
    [moveCamera]
  );

  const onRelease = useCallback((): void => {
    dragOriginRef.current = null;
  }, []);

  // A picture this viewport places itself opens fitted, rather than inheriting the last one's camera and opening
  // somewhere off screen.
  useEffect(() => {
    if (!isControlled) {
      setOwnState(PAN_ZOOM_FIT);
    }
  }, [isControlled, src]);

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
            transform: `translate(${transform.offsetX}px, ${transform.offsetY}px) scale(${transform.scale})`,
            imageRendering: transform.scale > 1 ? "pixelated" : "auto",
            userSelect: "none",
          }}
        />

        {hasControls ? (
          <ImageViewportControls
            scale={transform.scale}
            onZoomIn={onZoomIn}
            onZoomOut={onZoomOut}
            onActualSize={onActualSize}
            onFit={onFit}
          />
        ) : null}
      </Box>
    </Box>
  );
}
