import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useState } from "react";

import { describeTextureShape } from "@/core/assets/lib";
import { AssetTextureShape, TextureDescription } from "@/core/bindings/types/xrf-app";
import { TextureSurface } from "@/core/textures/components/preview/TextureSurface";
import {
  DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  describeTexturePreviewGap,
  ETexturePreviewMode,
  ITexturePreviewComparison,
  ITexturePreviewGap,
  ITexturePreviewOptions,
} from "@/core/textures/lib/texture-preview";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { PAN_ZOOM_FIT } from "@/lib/media/pan-zoom";
import { PanZoomController } from "@/lib/media/pan-zoom-controller";
import { Nullable } from "@/lib/types/general";

import { TextureImagePane } from "./TextureImagePane";
import { TexturePreviewFrame } from "./TexturePreviewFrame";

/** One texture is shown at a time, so its url lives under a fixed key and displaces the last one. */
const TEXTURE_PREVIEW_ASSET_KEY: string = "texture-preview";

/** The second picture of a comparison, under its own key so the two do not release each other's url. */
const TEXTURE_COMPARISON_ASSET_KEY: string = "texture-preview-comparison";

interface ITexturePreviewProps extends BaseComponentProps {
  /** What the toolbar is asking for. Defaulted, so the preview stands on its own outside the editor. */
  options?: ITexturePreviewOptions;
  /** Changes whenever the toolbar asks the lit body to put its camera and light back. */
  resetToken?: number;
  /**
   * Another encoding of this texture to show beside it, or null to show the file alone.
   *
   * Only the flat picture pairs: two lit bodies would be two scenes to light and orbit, and what a re-encode changes
   * is read off the texels rather than off the shading.
   */
  comparison?: Nullable<ITexturePreviewComparison>;
}

/**
 * The selected texture, flat or on a lit body, and optionally beside another encoding of itself.
 */
export function TexturePreview({
  "data-testid": dataTestId = "texture-preview",
  id,
  className,
  options = DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  resetToken = 0,
  comparison = null,
}: ITexturePreviewProps): ReactElement {
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  // Held here rather than in either pane, so a pair moves together: panning one picture to a corner and finding the
  // other still centred is the one thing a comparison must not do.
  const [panZoom] = useState<PanZoomController>(() => new PanZoomController());

  // The description of the texture being replaced is still here while the next one is read, so what says a read is in
  // progress is the async state rather than the absence of a description.
  const isReading: boolean = selectionService.selected.isLoading || selectionService.preview.isLoading;
  const description: Nullable<TextureDescription> = selectionService.selected.value;

  // Keyed on the texture rather than on either pane's url, which is what holding the camera for a pair means here: the
  // second picture is another encoding of the first, and re-encoding while zoomed into a block must not throw the
  // person back out to the fit.
  useEffect(() => panZoom.set(PAN_ZOOM_FIT), [panZoom, description?.reference]);

  if (selectionService.selected.error) {
    return (
      <ErrorState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"Could not read this texture"}
        description={selectionService.selected.error.message}
        onRetry={() => void selectionService.retry()}
      />
    );
  }

  if (!description && !isReading) {
    return (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"No texture open"}
        description={"Pick a texture in the tree to see it, and what its descriptor declares."}
      />
    );
  }

  if (isReading || !description) {
    return (
      <TexturePreviewFrame data-testid={dataTestId} id={id} className={className} caption={"Reading…"}>
        <DelayedProgress label={"Reading texture…"} />
      </TexturePreviewFrame>
    );
  }

  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;
  const gap: Nullable<ITexturePreviewGap> = describeTexturePreviewGap(
    options.mode,
    description.texture !== null,
    Boolean(shape && selectionService.preview.value)
  );

  const isImage: boolean = options.mode === ETexturePreviewMode.IMAGE;
  const caption: string = shape ? describeTextureShape(shape) : description.reference;

  if (comparison && isImage && !gap) {
    return (
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        sx={{ display: "flex", flexGrow: 1, gap: 1, minHeight: 0, minWidth: 0 }}
      >
        <TextureImagePane
          data-testid={"texture-image-pane-current"}
          caption={`On disk — ${caption}`}
          alt={description.reference}
          assetKey={TEXTURE_PREVIEW_ASSET_KEY}
          preview={selectionService.preview}
          shape={shape}
          controller={panZoom}
        />

        <TextureImagePane
          data-testid={"texture-image-pane-comparison"}
          caption={`Would write — ${comparison.label}`}
          alt={`${description.reference} as ${comparison.label}`}
          assetKey={TEXTURE_COMPARISON_ASSET_KEY}
          preview={comparison.preview}
          shape={shape}
          controller={panZoom}
          hasControls={false}
        />
      </Box>
    );
  }

  if (!gap && isImage) {
    return (
      <TextureImagePane
        data-testid={dataTestId}
        id={id}
        className={className}
        caption={caption}
        alt={description.reference}
        assetKey={TEXTURE_PREVIEW_ASSET_KEY}
        preview={selectionService.preview}
        shape={shape}
      />
    );
  }

  return (
    <TexturePreviewFrame
      data-testid={dataTestId}
      id={id}
      className={className}
      caption={caption}
      // The lit surface is a transparent canvas and needs the frame's checkerboard; a gap needs no ground at all.
      isCheckered={!gap}
    >
      {gap ? (
        <EmptyState title={gap.title} description={gap.description} />
      ) : (
        <TextureSurface options={options} resetToken={resetToken} />
      )}
    </TexturePreviewFrame>
  );
}
