import { IPanZoomTransform, toPanZoomTransform } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { ReactElement, useLayoutEffect, useRef } from "react";

import { IImageViewportView } from "@/core/ui/media/ImageViewport";

interface ILevelViewerMapAnnotationsOverlayProps {
  view: IImageViewportView;
  /** The annotations' SVG, as an object URL. */
  url: string;
}

/** A level map's annotations laid over it in its viewport, panned and zoomed with the map they are drawn for. */
export function LevelViewerMapAnnotationsOverlay({ view, url }: ILevelViewerMapAnnotationsOverlayProps): ReactElement {
  const imageRef = useRef<HTMLImageElement>(null);
  const { controller, content, viewport } = view;
  // The viewport hands a fresh size each render; the overlay follows its numbers, not its objects.
  const { width, height } = content;
  const { width: viewportWidth, height: viewportHeight } = viewport;

  useLayoutEffect(() => {
    function place(): void {
      const element: Nullable<HTMLImageElement> = imageRef.current;

      if (!element) {
        return;
      }

      const transform: IPanZoomTransform = toPanZoomTransform(
        controller.get(),
        { width, height },
        { width: viewportWidth, height: viewportHeight }
      );

      element.style.width = `${width}px`;
      element.style.height = `${height}px`;
      element.style.transform = `translate(${transform.offsetX}px, ${transform.offsetY}px) scale(${transform.scale})`;
    }

    place();

    return controller.subscribe(place);
  }, [controller, height, viewportHeight, viewportWidth, width]);

  return (
    <img
      ref={imageRef}
      className={"absolute top-0 left-0 max-w-none origin-top-left will-change-transform select-none"}
      alt={"Annotations"}
      src={url}
      draggable={false}
    />
  );
}
