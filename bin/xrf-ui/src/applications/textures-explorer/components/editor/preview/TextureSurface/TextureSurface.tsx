import { Box, SxProps, Theme } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { PointerEvent, ReactElement, useCallback, useEffect, useRef } from "react";

import {
  EMPTY_TEXTURE_SURFACE,
  ITextureSurfaceOptions,
  ITextureSurfaceTextures,
} from "@/applications/textures-explorer/lib/texture-surface";
import { TextureSurfaceService } from "@/applications/textures-explorer/services/surface";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { TextureSurfaceScene } from "./TextureSurfaceScene";

/** Covers the canvas while there is nothing on it to look at, without unmounting the scene beneath. */
const OVERLAY_STYLES: SxProps<Theme> = {
  alignItems: "center",
  bottom: 0,
  display: "flex",
  justifyContent: "center",
  left: 0,
  position: "absolute",
  right: 0,
  top: 0,
};

/** Where a light drag started, so each move swings by its own delta rather than the whole gesture. */
interface IDragOrigin {
  x: number;
  y: number;
}

interface ITextureSurfaceProps extends BaseComponentProps {
  options: ITextureSurfaceOptions;
  /** Changes whenever the toolbar asks for the camera and the light to go back where they started. */
  resetToken: number;
}

/**
 * The selected texture on a lit body, shaded the way the engine shades it.
 */
export function TextureSurface({
  "data-testid": dataTestId = "texture-surface",
  id,
  className,
  options,
  resetToken,
}: ITextureSurfaceProps): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);

  const containerRef = useRef<Nullable<HTMLDivElement>>(null);
  const sceneRef = useRef<Nullable<TextureSurfaceScene>>(null);
  const dragRef = useRef<Nullable<IDragOrigin>>(null);

  const description: Nullable<TextureDescription> = texturesService.selected.value;
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

  const onPointerUp = useCallback((event: PointerEvent<HTMLDivElement>): void => {
    if (!dragRef.current) {
      return;
    }

    dragRef.current = null;
    event.currentTarget.releasePointerCapture(event.pointerId);
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

  useEffect(() => sceneRef.current?.setTextures(textures), [textures]);

  useEffect(() => sceneRef.current?.setOptions(options), [options]);

  useEffect(() => {
    // Skipped at zero, which is the token before anyone has asked: a fresh scene is already where reset would put it.
    if (resetToken) {
      sceneRef.current?.reset();
    }
  }, [resetToken]);

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, position: "relative" }}
    >
      <Box
        ref={containerRef}
        sx={{
          flexGrow: 1,
          minHeight: 0,
          minWidth: 0,
          overflow: "hidden",
          visibility: isUploading || isUntextured ? "hidden" : "visible",
        }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
      />

      {isUploading ? (
        <Box sx={OVERLAY_STYLES}>
          <DelayedProgress />
        </Box>
      ) : null}

      {isUntextured ? (
        <Box sx={OVERLAY_STYLES}>
          <EmptyState
            title={"Nothing to lay on a surface"}
            description={"This file is a layout three.js cannot upload, so there is nothing here to light."}
          />
        </Box>
      ) : null}
    </Box>
  );
}
