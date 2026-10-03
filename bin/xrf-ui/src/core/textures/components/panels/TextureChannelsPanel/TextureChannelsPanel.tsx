import { useInjection } from "@wirestate/react";
import { Nullable, Optional } from "@xrf/types";
import { PointerEvent, ReactElement, RefCallback, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { ETextureBumpPlane, toTextureBumpPlane } from "@/core/textures/lib/texture-bump-plane";
import { ITextureBumpTexels, ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { ABSENT_VALUE } from "@/lib/format/number";

import {
  describeTextureTexel,
  ITextureTexelPosition,
  ITextureTexelReadout,
  toTextureTexelPosition,
} from "./texture-channel-readout";
import {
  describeTextureChannelsGap,
  ITextureChannelTile,
  TEXTURE_CHANNEL_TILES,
  toTextureChannelAspect,
} from "./TextureChannelsPanel.utils";

/**
 * @param texels - Both halves of the pair on the cpu.
 * @param plane - The plane wanted.
 * @returns The plane drawn into a canvas a texel a pixel, for a tile to scale; empty where no 2d context is had.
 */
function toPlaneCanvas(texels: ITextureBumpTexels, plane: ETextureBumpPlane): HTMLCanvasElement {
  const canvas: HTMLCanvasElement = document.createElement("canvas");
  const context: Nullable<CanvasRenderingContext2D> = canvas.getContext("2d");

  canvas.width = texels.bump.width;
  canvas.height = texels.bump.height;

  if (context) {
    const image: ImageData = context.createImageData(canvas.width, canvas.height);

    image.data.set(toTextureBumpPlane(texels, plane));
    context.putImageData(image, 0, 0);
  }

  return canvas;
}

/**
 * The two planes of the selected texture's bump pair, and the three values the engine reconstructs from them.
 */
export function TextureChannelsPanel({
  "data-testid": dataTestId = "texture-channels-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);

  const tilesRef = useRef<Map<ETextureBumpPlane, HTMLCanvasElement>>(new Map());

  const [position, setPosition] = useState<Nullable<ITextureTexelPosition>>(null);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const files: Nullable<ITextureSurfaceFiles> = surfaceService.files.value;
  const texels: Nullable<ITextureBumpTexels> = files?.bump ?? null;
  const isReading: boolean = surfaceService.files.isLoading;
  const gap: Nullable<string> = describeTextureChannelsGap(description, files, isReading);
  const readout: Nullable<ITextureTexelReadout> = texels && position ? describeTextureTexel(texels, position) : null;
  // Laid out from the pair's own proportions, so a plane is never shown stretched into a square.
  const aspect: string = toTextureChannelAspect(files);

  // Each plane built once a pair, a texel a pixel at the pair's own size; a tile draws it scaled to its own.
  const planes: ReadonlyMap<ETextureBumpPlane, HTMLCanvasElement> = useMemo(
    () =>
      texels
        ? new Map(TEXTURE_CHANNEL_TILES.map((tile) => [tile.plane, toPlaneCanvas(texels, tile.plane)]))
        : new Map(),
    [texels]
  );

  const draw = useCallback(() => {
    const ratio: number = window.devicePixelRatio;

    for (const [plane, tile] of tilesRef.current) {
      const source: Optional<HTMLCanvasElement> = planes.get(plane);
      const context: Nullable<CanvasRenderingContext2D> = tile.getContext("2d");
      const width: number = Math.round(tile.clientWidth * ratio);
      const height: number = Math.round(tile.clientHeight * ratio);

      if (!context || !width || !height) {
        continue;
      }

      tile.width = width;
      tile.height = height;

      if (source) {
        context.drawImage(source, 0, 0, width, height);
      }
    }
  }, [planes]);

  useEffect(() => draw(), [draw]);

  // A panel is resized by hand and by the window, and a tile that is not redrawn afterwards keeps the last size it was
  // copied at, stretched.
  useEffect(() => {
    const observer: ResizeObserver = new ResizeObserver(() => draw());

    for (const tile of tilesRef.current.values()) {
      observer.observe(tile);
    }

    return () => observer.disconnect();
  }, [draw, gap]);

  // One callback per plane for the panel's life: a new one each render would detach and attach every tile each time.
  const registers: ReadonlyMap<ETextureBumpPlane, RefCallback<HTMLCanvasElement>> = useMemo(
    () =>
      new Map(
        TEXTURE_CHANNEL_TILES.map((tile: ITextureChannelTile) => [
          tile.plane,
          (canvas: Nullable<HTMLCanvasElement>): void => {
            if (canvas) {
              tilesRef.current.set(tile.plane, canvas);
            } else {
              tilesRef.current.delete(tile.plane);
            }
          },
        ])
      ),
    []
  );

  const onHover = useCallback(
    (event: PointerEvent<HTMLElement>): void => {
      if (!texels) {
        return;
      }

      const bounds: DOMRect = event.currentTarget.getBoundingClientRect();

      setPosition(
        toTextureTexelPosition(
          texels,
          (event.clientX - bounds.left) / bounds.width,
          (event.clientY - bounds.top) / bounds.height
        )
      );
    },
    [texels]
  );

  return (
    <EditorPanel data-testid={dataTestId} id={id} className={className} title={"Channels"}>
      {gap ? (
        <EditorPanelEmpty label={gap} />
      ) : (
        <>
          <EditorPanelSection
            data-testid={"texture-channels-readout"}
            title={"Texel"}
            caption={texels ? "Under the pointer, as stored and as reconstructed" : undefined}
            isFirst
          >
            {texels ? (
              <>
                <EditorPanelProperty label={"At"} value={readout?.position ?? ABSENT_VALUE} />
                <EditorPanelProperty label={"Bump"} value={readout?.bump ?? ABSENT_VALUE} />
                <EditorPanelProperty label={"Bump#"} value={readout?.companion ?? ABSENT_VALUE} />
                <EditorPanelProperty label={"Normal"} value={readout?.normal ?? ABSENT_VALUE} />
                <EditorPanelProperty label={"Gloss"} value={readout?.gloss ?? ABSENT_VALUE} />
                <EditorPanelProperty label={"Height"} value={readout?.height ?? ABSENT_VALUE} />
              </>
            ) : (
              <EditorPanelProperty
                label={"Readout"}
                value={"Unavailable: this pair is stored as blocks rather than as plain texels"}
              />
            )}
          </EditorPanelSection>

          {TEXTURE_CHANNEL_TILES.map((tile: ITextureChannelTile) => (
            <EditorPanelSection key={tile.plane} title={tile.label} caption={tile.caption}>
              <div
                className={"relative w-full checkerboard"}
                style={{ aspectRatio: aspect }}
                onPointerMove={onHover}
                onPointerLeave={() => setPosition(null)}
              >
                <canvas
                  ref={registers.get(tile.plane)}
                  data-testid={`texture-channel-${tile.plane}`}
                  aria-label={tile.label}
                  className={"block size-full"}
                  role={"img"}
                />
              </div>
            </EditorPanelSection>
          ))}
        </>
      )}
    </EditorPanel>
  );
}
