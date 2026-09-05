import { Box, Paper, SxProps, Theme, ToggleButton, ToggleButtonGroup, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { PointerEvent, ReactElement, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { EMPTY_TEXTURE_SURFACE, ITextureSurfaceTextures } from "@/applications/textures-explorer/lib/texture-surface";
import { TextureSurfaceService } from "@/applications/textures-explorer/services/surface";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ITextureSurfaceOptions, TextureSurfaceScene } from "./texture-surface-scene";
import { ETextureSurfaceShape } from "./texture-surface.utils";

/** How many times a texture may be repeated across the body, in the steps a seam is actually judged at. */
const TILING_STEPS: ReadonlyArray<number> = [1, 2, 4];

const SHAPE_LABELS: Record<ETextureSurfaceShape, string> = {
  [ETextureSurfaceShape.PLANE]: "Plane",
  [ETextureSurfaceShape.SPHERE]: "Sphere",
  [ETextureSurfaceShape.CUBE]: "Cube",
};

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

/**
 * The selected texture on a lit body, shaded the way the engine shades it.
 */
export function TextureSurface({
  "data-testid": dataTestId = "texture-surface",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);

  const containerRef = useRef<Nullable<HTMLDivElement>>(null);
  const sceneRef = useRef<Nullable<TextureSurfaceScene>>(null);
  const dragRef = useRef<Nullable<IDragOrigin>>(null);

  const [shape, setShape] = useState<ETextureSurfaceShape>(ETextureSurfaceShape.PLANE);
  const [tiling, setTiling] = useState<number>(1);
  const [isBumped, setBumped] = useState<boolean>(true);

  const description: Nullable<TextureDescription> = texturesService.selected.value;
  const textures: ITextureSurfaceTextures = surfaceService.textures.value ?? EMPTY_TEXTURE_SURFACE;
  const hasBump: boolean = textures.bump !== null;
  const isUploading: boolean = surfaceService.textures.isLoading;
  // Keyed on which texture was uploaded, so this says "nothing to draw" only once an upload has answered for the one
  // on screen, rather than during the frame between choosing a texture and asking for its files.
  const isUntextured: boolean =
    !isUploading && textures.base === null && surfaceService.uploaded === description?.reference;

  const options: ITextureSurfaceOptions = useMemo(() => ({ isBumped, shape, tiling }), [isBumped, shape, tiling]);

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

  // Uploaded while this surface is on screen and dropped when it leaves, so the flat preview costs no gpu memory.
  useEffect(() => {
    if (description) {
      void surfaceService.load(description);
    } else {
      void surfaceService.clear();
    }
  }, [surfaceService, description]);

  useEffect(() => () => void surfaceService.clear(), [surfaceService]);

  useEffect(() => sceneRef.current?.setTextures(textures), [textures]);

  useEffect(() => sceneRef.current?.setOptions(options), [options]);

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

      <Paper
        elevation={3}
        sx={{
          bottom: 12,
          display: "flex",
          gap: 1,
          left: "50%",
          padding: 0.75,
          position: "absolute",
          transform: "translateX(-50%)",
        }}
      >
        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={shape}
          aria-label={"Surface shape"}
          onChange={(_, next: Nullable<ETextureSurfaceShape>) => next && setShape(next)}
        >
          {Object.values(ETextureSurfaceShape).map((value: ETextureSurfaceShape) => (
            <ToggleButton key={value} value={value}>
              {SHAPE_LABELS[value]}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>

        <ToggleButtonGroup
          exclusive
          size={"small"}
          value={tiling}
          aria-label={"Tiling"}
          onChange={(_, next: Nullable<number>) => next && setTiling(next)}
        >
          {TILING_STEPS.map((value: number) => (
            <ToggleButton key={value} value={value} aria-label={`Tile ${value} by ${value}`}>
              {`${value}×`}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>

        <Tooltip
          title={
            hasBump ? "Shade with the declared pair, or draw the same body flat" : "This texture declares no bump pair"
          }
        >
          <Box component={"span"} sx={{ display: "flex" }}>
            <ToggleButton
              size={"small"}
              value={"bump"}
              selected={isBumped && hasBump}
              disabled={!hasBump}
              onChange={() => setBumped(!isBumped)}
            >
              Bump
            </ToggleButton>
          </Box>
        </Tooltip>

        <Tooltip title={"Put the camera and the light back. Shift and drag moves the light"}>
          <ToggleButton size={"small"} value={"reset"} onClick={() => sceneRef.current?.reset()}>
            Reset
          </ToggleButton>
        </Tooltip>
      </Paper>
    </Box>
  );
}
