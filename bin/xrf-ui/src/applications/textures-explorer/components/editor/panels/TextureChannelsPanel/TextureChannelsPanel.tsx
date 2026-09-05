import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { PointerEvent, ReactElement, useCallback, useEffect, useRef, useState } from "react";

import { ITextureBumpTexels, ITextureSurfaceTextures } from "@/applications/textures-explorer/lib/texture-surface";
import { TextureSurfaceService } from "@/applications/textures-explorer/services/surface";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { IMAGE_CHECKERBOARD } from "@/core/ui/media/media.styles";
import { VisualPanel } from "@/core/visuals/components/panels/VisualPanel";
import { VisualPanelEmpty } from "@/core/visuals/components/panels/VisualPanelEmpty";
import { VisualPanelRow } from "@/core/visuals/components/panels/VisualPanelRow";
import { VisualPanelSection } from "@/core/visuals/components/panels/VisualPanelSection";
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
  const texturesService: TexturesService = useInjection(TexturesService);
  const surfaceService: TextureSurfaceService = useInjection(TextureSurfaceService);

  const rendererRef = useRef<Nullable<TextureChannelRenderer>>(null);
  const tilesRef = useRef<Map<EVisualBumpView, HTMLCanvasElement>>(new Map());

  const [position, setPosition] = useState<Nullable<ITextureTexelPosition>>(null);

  const description: Nullable<TextureDescription> = texturesService.selected.value;
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
    <VisualPanel data-testid={dataTestId} id={id} className={className} title={"Channels"}>
      {gap ? (
        <VisualPanelEmpty label={gap} />
      ) : (
        <>
          <VisualPanelSection
            data-testid={"texture-channels-readout"}
            title={"Texel"}
            caption={texels ? "Under the pointer, as stored and as reconstructed" : undefined}
            isFirst
          >
            {readout ? (
              <>
                <VisualPanelRow label={"At"} value={readout.position} />
                <VisualPanelRow label={"Bump"} value={readout.bump} />
                <VisualPanelRow label={"Bump#"} value={readout.companion} />
                <VisualPanelRow label={"Normal"} value={readout.normal} />
                <VisualPanelRow label={"Gloss"} value={readout.gloss} />
                <VisualPanelRow label={"Height"} value={readout.height} />
              </>
            ) : (
              <VisualPanelRow
                label={"Readout"}
                value={
                  texels ? "Point at a tile" : "Unavailable: this pair is stored as blocks rather than as plain texels"
                }
              />
            )}
          </VisualPanelSection>

          {TEXTURE_CHANNEL_TILES.map((tile: ITextureChannelTile) => (
            <VisualPanelSection key={tile.view} title={tile.label} caption={tile.caption}>
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
            </VisualPanelSection>
          ))}
        </>
      )}
    </VisualPanel>
  );
}
