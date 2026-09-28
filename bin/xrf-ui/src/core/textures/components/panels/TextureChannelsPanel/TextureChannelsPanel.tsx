import { useInjection } from "@wirestate/react";
import { ERendererBumpPlane } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import { PointerEvent, ReactElement, RefCallback, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import {
  EditorPanel,
  EditorPanelEmpty,
  EditorPanelProperty,
  EditorPanelSection,
} from "@/core/shell/editor/EditorPanel";
import { ITextureBumpTexels, ITextureSurfaceFiles } from "@/core/textures/lib/texture-surface";
import { TextureRenderService } from "@/core/textures/services/render";
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
 * The two planes of the selected texture's bump pair, and the three values the engine reconstructs from them.
 */
export function TextureChannelsPanel({
  "data-testid": dataTestId = "texture-channels-panel",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);
  const renderService: TextureRenderService = useInjection(TextureRenderService);

  const tilesRef = useRef<Map<ERendererBumpPlane, HTMLCanvasElement>>(new Map());
  // By plane, and only ever counted up: a tile mounted again must not take an answer asked for by the one before it.
  const requestsRef = useRef<Map<ERendererBumpPlane, number>>(new Map());

  const [position, setPosition] = useState<Nullable<ITextureTexelPosition>>(null);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const files: Nullable<ITextureSurfaceFiles> = surfaceService.files.value;
  const texels: Nullable<ITextureBumpTexels> = surfaceService.bumpTexels;
  const isReading: boolean = surfaceService.files.isLoading;
  const failure: Nullable<string> = renderService.failure;
  const gap: Nullable<string> = failure
    ? `The renderer stopped: ${failure}`
    : describeTextureChannelsGap(description, files, isReading);
  const readout: Nullable<ITextureTexelReadout> = texels && position ? describeTextureTexel(texels, position) : null;
  // Laid out from the pair's own proportions, so a plane is never shown stretched into a square.
  const aspect: string = toTextureChannelAspect(files);

  // Drawn by the texture's renderer at each tile's own size, on its thread, and copied in when it answers. Nothing
  // animates: a tile is drawn when the pair changes and when the tile resizes.
  const draw = useCallback(() => {
    const ratio: number = window.devicePixelRatio;

    for (const [plane, tile] of tilesRef.current) {
      const width: number = Math.round(tile.clientWidth * ratio);
      const height: number = Math.round(tile.clientHeight * ratio);
      const request: number = (requestsRef.current.get(plane) ?? 0) + 1;

      if (!width || !height) {
        continue;
      }

      requestsRef.current.set(plane, request);

      renderService.captureBumpPlane(plane, width, height).then(
        (image: Nullable<ImageBitmap>) => {
          if (requestsRef.current.get(plane) !== request || tilesRef.current.get(plane) !== tile || !image) {
            image?.close();

            return;
          }

          tile.width = image.width;
          tile.height = image.height;
          tile.getContext("2d")?.drawImage(image, 0, 0);
          image.close();
        },
        // A renderer that failed draws no tile; the panel says why instead.
        () => undefined
      );
    }
  }, [renderService]);

  useEffect(() => {
    draw();
  }, [draw, files]);

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
  const registers: ReadonlyMap<ERendererBumpPlane, RefCallback<HTMLCanvasElement>> = useMemo(
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
                className={"w-full checkerboard"}
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
