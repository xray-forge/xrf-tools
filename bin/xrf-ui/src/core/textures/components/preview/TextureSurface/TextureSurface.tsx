import { useInjection } from "@wirestate/react";
import { PointerEvent, ReactElement, useCallback, useEffect, useRef } from "react";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureSurfaceOptions,
  ITextureSurfaceTextures,
} from "@/core/textures/lib/texture-surface";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ViewportControls } from "@/core/ui/media/ViewportControls";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { DOLLY_STEP } from "@/lib/media/orbit-dolly";
import { Nullable } from "@/lib/types/general";

import { TextureSurfaceScene } from "./TextureSurfaceScene";

/** Where a light drag started, so each move swings by its own delta rather than the whole gesture. */
interface IDragOrigin {
  x: number;
  y: number;
}

interface ITextureSurfaceProps extends BaseComponentProps {
  options: ITextureSurfaceOptions;
}

/**
 * The selected texture on a lit body, shaded the way the engine shades it.
 */
export function TextureSurface({
  "data-testid": dataTestId = "texture-surface",
  id,
  className,
  options,
}: ITextureSurfaceProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);

  const containerRef = useRef<Nullable<HTMLDivElement>>(null);
  const sceneRef = useRef<Nullable<TextureSurfaceScene>>(null);
  const dragRef = useRef<Nullable<IDragOrigin>>(null);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const textures: ITextureSurfaceTextures = surfaceService.textures.value ?? EMPTY_TEXTURE_SURFACE;
  const isUploading: boolean = surfaceService.textures.isLoading;
  // Keyed on which texture was uploaded, so this says "nothing to draw" only once an upload has answered for the one
  // on screen, rather than during the frame between choosing a texture and asking for its files.
  const isUntextured: boolean =
    !isUploading && textures.base === null && surfaceService.uploaded === description?.reference;

  const onPointerDown = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    // Shift is what separates moving the light from orbiting the camera, since both are a drag over the same body.
    if (!event.shiftKey) {
      return;
    }

    dragRef.current = { x: event.clientX, y: event.clientY };
    event.currentTarget.setPointerCapture(event.pointerId);
  }, []);

  const onPointerMove = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    const origin: Nullable<IDragOrigin> = dragRef.current;

    if (!origin) {
      return;
    }

    sceneRef.current?.dragLight(event.clientX - origin.x, event.clientY - origin.y);
    dragRef.current = { x: event.clientX, y: event.clientY };
  }, []);

  const onPointerEnd = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    // The child canvas also captures pointers through OrbitControls; its capture loss is not ours.
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

  useEffect(() => {
    const scene: TextureSurfaceScene = new TextureSurfaceScene();

    sceneRef.current = scene;

    if (containerRef.current) {
      scene.mount(containerRef.current);
    }

    return () => {
      scene.dispose();
      sceneRef.current = null;
    };
  }, []);

  const onZoomIn = useCallback((): void => sceneRef.current?.dolly(1 / DOLLY_STEP), []);

  const onZoomOut = useCallback((): void => sceneRef.current?.dolly(DOLLY_STEP), []);

  const onReset = useCallback((): void => sceneRef.current?.reset(), []);

  useEffect(() => sceneRef.current?.setTextures(textures), [textures]);

  useEffect(() => sceneRef.current?.setOptions(options), [options]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("relative flex min-h-0 min-w-0 grow", className)}>
      <div
        ref={containerRef}
        className={cn("min-h-0 min-w-0 grow overflow-hidden", isUploading || isUntextured ? "invisible" : null)}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerEnd}
        onPointerCancel={onPointerEnd}
        onLostPointerCapture={onPointerEnd}
      />

      {isUploading ? (
        <div className={"absolute inset-0 flex items-center justify-center"}>
          <DelayedProgress label={"Preparing texture surface…"} />
        </div>
      ) : null}

      {isUntextured ? (
        <div className={"absolute inset-0 flex items-center justify-center"}>
          <EmptyState
            title={"Nothing to lay on a surface"}
            description={"This file is a layout three.js cannot upload, so there is nothing here to light."}
          />
        </div>
      ) : null}

      {isUploading || isUntextured ? null : (
        <ViewportControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onReset={onReset} />
      )}
    </div>
  );
}
