import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { PointerEvent, ReactElement, useCallback, useEffect, useRef, useState } from "react";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EditorPanel, EditorPanelEmpty, EditorPanelRow, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { ITextureBumpTexels, ITextureSurfaceTextures } from "@/core/textures/lib/texture-surface";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import { EVisualBumpView } from "@/core/visuals/lib/visual-bump-channels";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import {
  describeTextureTexel,
  ITextureTexelPosition,
  ITextureTexelReadout,
  toTextureTexelPosition,
} from "./texture-channel-readout";
import { TextureChannelRenderer } from "./TextureChannelRenderer";
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

  const rendererRef = useRef<Nullable<TextureChannelRenderer>>(null);
  const tilesRef = useRef<Map<EVisualBumpView, HTMLCanvasElement>>(new Map());

  const [position, setPosition] = useState<Nullable<ITextureTexelPosition>>(null);

  const description: Nullable<TextureDescription> = selectionService.selected.value;
  const textures: Nullable<ITextureSurfaceTextures> = surfaceService.textures.value;
  const texels: Nullable<ITextureBumpTexels> = surfaceService.bumpTexels;
  const isUploading: boolean = surfaceService.textures.isLoading;
  const gap: Nullable<string> = describeTextureChannelsGap(description, textures, isUploading);
  const readout: Nullable<ITextureTexelReadout> = texels && position ? describeTextureTexel(texels, position) : null;
  // Laid out from the pair's own proportions, so a plane is never shown stretched into a square.
  const aspect: string = toTextureChannelAspect(textures);

  const draw = useCallback(() => {
    const renderer: Nullable<TextureChannelRenderer> = rendererRef.current;

    if (!renderer) {
      return;
    }

    for (const [view, tile] of tilesRef.current) {
      renderer.draw(view, tile);
    }
  }, []);

  useEffect(() => {
    const renderer: TextureChannelRenderer = new TextureChannelRenderer();

    rendererRef.current = renderer;

    return () => {
      renderer.dispose();
      rendererRef.current = null;
    };
  }, []);

  useEffect(() => {
    rendererRef.current?.setTextures(textures?.bump ?? null);
    draw();
  }, [draw, textures]);

  // A panel is resized by hand and by the window, and a tile that is not redrawn afterwards keeps the last size it was
  // copied at, stretched.
  useEffect(() => {
    const observer: ResizeObserver = new ResizeObserver(() => draw());

    for (const tile of tilesRef.current.values()) {
      observer.observe(tile);
    }

    return () => observer.disconnect();
  }, [draw, gap]);

  const registerTile = useCallback(
    (view: EVisualBumpView) => (tile: Nullable<HTMLCanvasElement>) => {
      if (tile) {
        tilesRef.current.set(view, tile);
      } else {
        tilesRef.current.delete(view);
      }
    },
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
            {readout ? (
              <>
                <EditorPanelRow label={"At"} value={readout.position} />
                <EditorPanelRow label={"Bump"} value={readout.bump} />
                <EditorPanelRow label={"Bump#"} value={readout.companion} />
                <EditorPanelRow label={"Normal"} value={readout.normal} />
                <EditorPanelRow label={"Gloss"} value={readout.gloss} />
                <EditorPanelRow label={"Height"} value={readout.height} />
              </>
            ) : (
              <EditorPanelRow
                label={"Readout"}
                value={
                  texels ? "Point at a tile" : "Unavailable: this pair is stored as blocks rather than as plain texels"
                }
              />
            )}
          </EditorPanelSection>

          {TEXTURE_CHANNEL_TILES.map((tile: ITextureChannelTile) => (
            <EditorPanelSection key={tile.view} title={tile.label} caption={tile.caption}>
              <Box
                sx={[{ aspectRatio: aspect, width: "100%" }, IMAGE_CHECKERBOARD]}
                onPointerMove={onHover}
                onPointerLeave={() => setPosition(null)}
              >
                <Box
                  component={"canvas"}
                  data-testid={`texture-channel-${tile.view}`}
                  ref={registerTile(tile.view)}
                  sx={{ display: "block", height: "100%", width: "100%" }}
                />
              </Box>
            </EditorPanelSection>
          ))}
        </>
      )}
    </EditorPanel>
  );
}
