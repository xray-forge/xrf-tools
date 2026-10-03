import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { PointerEvent, ReactElement, useCallback, useRef } from "react";

import { RenderFailureCover, RenderFrameReadout } from "@/core/render/components/overlay";
import { RenderSurface } from "@/core/render/components/RenderSurface";
import { DOLLY_STEP } from "@/core/render/lib/contract/renderer-camera-command";
import { TextureRenderService } from "@/core/textures/services/render";
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
  const renderService: TextureRenderService = useInjection(TextureRenderService);

  const dragRef = useRef<Nullable<IDragOrigin>>(null);

  // Said once the renderer has read the texture on screen and could not lay it on the body.
  const failure: Nullable<string> = renderService.baseFailure;

  const onPointerDown = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    // Shift is what separates moving the light from turning the camera, since both are a drag over the same body; taken
    // before the viewport hears it, which would pan.
    if (!event.shiftKey) {
      return;
    }

    event.stopPropagation();
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
        className={cn("min-h-0 min-w-0 grow overflow-hidden", failure ? "invisible" : null)}
        onPointerDownCapture={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerEnd}
        onPointerCancel={onPointerEnd}
        onLostPointerCapture={onPointerEnd}
      >
        <RenderSurface host={renderService} />
      </div>

      {failure ? (
        <div className={"absolute inset-0 flex items-center justify-center"}>
          <EmptyState title={"Nothing to lay on a surface"} description={failure} />
        </div>
      ) : null}

      {failure || renderService.failure ? null : (
        <>
          <RenderFrameReadout cost={renderService.frameCost} timings={renderService.timings} />

          <ViewportControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onReset={onReset} />
        </>
      )}

      <RenderFailureCover failure={renderService.failure} />
    </div>
  );
}
