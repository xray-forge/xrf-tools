import { useInjection } from "@wirestate/react";
import { DOLLY_STEP } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { PointerEvent, ReactElement, useCallback, useRef } from "react";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { RenderFrameReadout } from "@/core/render/components/overlay";
import { RenderSurface } from "@/core/render/components/RenderSurface";
import { EMPTY_TEXTURE_SURFACE, ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";
import { TextureRenderService } from "@/core/textures/services/render";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ViewportControls } from "@/core/ui/media/ViewportControls";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Where a light drag started, so each move swings by its own delta rather than the whole gesture. */
interface IDragOrigin {
  x: number;
  y: number;
}

/**
 * The selected texture on a lit body, shaded the way the engine shades it.
 */
export function TextureSurface({
  "data-testid": dataTestId = "texture-surface",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);
  const renderService: TextureRenderService = useInjection(TextureRenderService);

  const dragRef = useRef<Nullable<IDragOrigin>>(null);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const files: ITextureSurfaceFiles = surfaceService.files.value ?? EMPTY_TEXTURE_SURFACE;
  const isUploading: boolean = surfaceService.files.isLoading;
  // Keyed on which texture was uploaded, so this says "nothing to draw" only once an upload has answered for the one
  // on screen, rather than during the frame between choosing a texture and asking for its files.
  const isUntextured: boolean =
    !isUploading && files.base === null && surfaceService.reference === description?.reference;

  const onPointerDown = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    // Shift is what separates moving the light from orbiting the camera, since both are a drag over the same body.
    if (!event.shiftKey) {
      return;
    }

    dragRef.current = { x: event.clientX, y: event.clientY };
    event.currentTarget.setPointerCapture(event.pointerId);
  }, []);

  const onPointerMove = useCallback(
    (event: PointerEvent<HTMLDivElement>): void => {
      const origin: Nullable<IDragOrigin> = dragRef.current;

      if (!origin) {
        return;
      }

      renderService.dragLight(event.clientX - origin.x, event.clientY - origin.y);

      dragRef.current = { x: event.clientX, y: event.clientY };
    },
    [renderService]
  );

  const onPointerEnd = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    if (event.type === "lostpointercapture" && event.target !== event.currentTarget) {
      return;
    }

    if (!dragRef.current) {
      return;
    }

    dragRef.current = null;

    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  }, []);

  const onZoomIn = useCallback((): void => renderService.dolly(1 / DOLLY_STEP), [renderService]);

  const onZoomOut = useCallback((): void => renderService.dolly(DOLLY_STEP), [renderService]);

  const onReset = useCallback((): void => renderService.reset(), [renderService]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("relative flex min-h-0 min-w-0 grow", className)}>
      <div
        className={cn("min-h-0 min-w-0 grow overflow-hidden", isUploading || isUntextured ? "invisible" : null)}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerEnd}
        onPointerCancel={onPointerEnd}
        onLostPointerCapture={onPointerEnd}
      >
        <RenderSurface host={renderService} />
      </div>

      {isUploading ? (
        <div className={"absolute inset-0 flex items-center justify-center"}>
          <DelayedProgress label={"Preparing texture surface…"} />
        </div>
      ) : null}

      {isUntextured ? (
        <div className={"absolute inset-0 flex items-center justify-center"}>
          <EmptyState
            title={"Nothing to lay on a surface"}
            description={
              "This file is a layout neither the renderer nor the backend can read, so there is nothing to light."
            }
          />
        </div>
      ) : null}

      {isUploading || isUntextured ? null : (
        <>
          <RenderFrameReadout cost={renderService.frameCost} />

          <ViewportControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onReset={onReset} />
        </>
      )}
    </div>
  );
}
