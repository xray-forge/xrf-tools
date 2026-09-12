import { Box } from "@mui/material";
import {
  MouseEvent,
  ReactElement,
  RefCallback,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  useSyncExternalStore,
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
import { PanZoomController } from "@/lib/media/pan-zoom-controller";
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
   * The camera to look through, for two viewports that must show the same part of two pictures. A viewport given none
   * keeps one of its own.
   */
  controller?: PanZoomController;
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
  controller: sharedController,
  hasControls = true,
}: IImageViewportProps): ReactElement {
  const [viewportRef, measured]: [RefCallback<HTMLDivElement>, Nullable<IElementSize>] =
    useElementSize<HTMLDivElement>();

  const imageRef = useRef<Nullable<HTMLImageElement>>(null);
  const dragOriginRef = useRef<Nullable<IPanZoomPoint>>(null);

  const [ownController] = useState<PanZoomController>(() => new PanZoomController());
  const controller: PanZoomController = sharedController ?? ownController;

  /** Puts the picture where the camera looks, which is everything a pan has to do. */
  const place = useCallback((): void => {
    const element: Nullable<HTMLImageElement> = imageRef.current;

    if (!element) {
      return;
    }

    const transform: IPanZoomTransform = toPanZoomTransform(
      controller.get(),
      { width, height },
      measured ?? UNMEASURED
    );

    element.style.transform = `translate(${transform.offsetX}px, ${transform.offsetY}px) scale(${transform.scale})`;
    // Nearest neighbour only above one to one, where the texels themselves are what is being looked at.
    element.style.imageRendering = transform.scale > 1 ? "pixelated" : "auto";
  }, [controller, height, measured, width]);

  const subscribe = useCallback((listener: () => void): (() => void) => controller.subscribe(listener), [controller]);

  // Read rather than held in state: a pan leaves the magnification alone and so renders nothing at all, and a zoom
  // changes one number and renders the bar that shows it.
  const scale: number = useSyncExternalStore(
    subscribe,
    (): number => toPanZoomTransform(controller.get(), { width, height }, measured ?? UNMEASURED).scale
  );

  /** Moves whichever camera the current state resolves to, and leaves the viewport looking through the result. */
  const moveCamera = useCallback(
    (move: (camera: IPanZoomCamera) => IPanZoomCamera): void => {
      controller.set((current: IPanZoomState) =>
        toManualPanZoom(move(resolvePanZoomCamera(current, { width, height }, measured ?? UNMEASURED)))
      );
    },
    [controller, height, measured, width]
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

  const onFit = useCallback((): void => controller.set(PAN_ZOOM_FIT), [controller]);

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

  useLayoutEffect(() => {
    place();

    return controller.subscribe(place);
  }, [controller, place]);

  // A picture this viewport places itself opens fitted, rather than inheriting the last one's camera and opening
  // somewhere off screen. A shared camera is reset by whoever owns it, since it outlives either picture on it.
  useEffect(() => {
    if (!sharedController) {
      ownController.set(PAN_ZOOM_FIT);
    }
  }, [ownController, sharedController, src]);

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
          ref={imageRef}
          component={"img"}
          alt={alt}
          src={src}
          draggable={false}
          sx={{
            position: "absolute",
            left: 0,
            top: 0,
            transformOrigin: "0 0",
            willChange: "transform",
            userSelect: "none",
          }}
        />

        {hasControls ? (
          <ImageViewportControls
            scale={scale}
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
