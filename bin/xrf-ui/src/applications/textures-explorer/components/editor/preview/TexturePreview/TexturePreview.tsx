import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useMemo, useState } from "react";

import { TextureSurface } from "@/applications/textures-explorer/components/editor/preview/TextureSurface";
import {
  DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  describeTexturePreviewGap,
  ETexturePreviewMode,
  ITexturePreviewGap,
  ITexturePreviewOptions,
} from "@/applications/textures-explorer/lib/texture-preview";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { describeTextureShape } from "@/core/assets/lib";
import { AssetService } from "@/core/assets/services";
import { AssetTextureShape, TextureDescription } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { TexturePreviewFrame } from "./TexturePreviewFrame";

/** One texture is shown at a time, so its url lives under a fixed key and displaces the last one. */
const TEXTURE_PREVIEW_ASSET_KEY: string = "texture-preview";

interface ITexturePreviewProps extends BaseComponentProps {
  /** What the toolbar is asking for. Defaulted, so the preview stands on its own outside the editor. */
  options?: ITexturePreviewOptions;
  /** Changes whenever the toolbar asks the lit body to put its camera and light back. */
  resetToken?: number;
}

/**
 * The selected texture, flat or on a lit body.
 */
export function TexturePreview({
  "data-testid": dataTestId = "texture-preview",
  id,
  className,
  options = DEFAULT_TEXTURE_PREVIEW_OPTIONS,
  resetToken = 0,
}: ITexturePreviewProps): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);
  const assetService: AssetService = useInjection(AssetService);

  const [url, setUrl] = useState<Nullable<string>>(null);

  // The description of the texture being replaced is still here while the next one is read, so what says a read is in
  // progress is the loadable rather than the absence of a description.
  const isReading: boolean = texturesService.selected.isLoading || texturesService.preview.isLoading;
  const description: Nullable<TextureDescription> = texturesService.selected.value;
  const bytes: Nullable<ArrayBuffer> = texturesService.preview.value;

  // Blobbed from the view rather than its buffer, so a byte offset cannot silently widen the picture.
  const blob: Nullable<Blob> = useMemo(() => (bytes ? new Blob([bytes], { type: "image/png" }) : null), [bytes]);

  useEffect(() => {
    setUrl(blob ? assetService.swap(TEXTURE_PREVIEW_ASSET_KEY, blob) : null);
  }, [assetService, blob]);

  if (texturesService.selected.error) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={"Could not read this texture"}
        description={texturesService.selected.error.message}
        action={
          <Button variant={"outlined"} onClick={() => void texturesService.retrySelected()}>
            Retry
          </Button>
        }
      />
    );
  }

  if (!description && !isReading) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={"No texture open"}
        description={"Pick a texture in the tree to see it, and what its descriptor declares."}
      />
    );
  }

  if (isReading || !description) {
    return (
      <TexturePreviewFrame data-testid={dataTestId} id={id} className={className} caption={"Reading…"}>
        <DelayedProgress />
      </TexturePreviewFrame>
    );
  }

  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;
  const gap: Nullable<ITexturePreviewGap> = describeTexturePreviewGap(
    options.mode,
    description.texture !== null,
    Boolean(shape && url)
  );

  // The flat viewport draws its own checkerboard; the lit surface is a transparent canvas and needs the frame's.
  const isCheckered: boolean = !gap && options.mode === ETexturePreviewMode.SURFACE;

  return (
    <TexturePreviewFrame
      data-testid={dataTestId}
      id={id}
      className={className}
      caption={shape ? describeTextureShape(shape) : description.reference}
      isCheckered={isCheckered}
    >
      {gap ? <EmptyState title={gap.title} description={gap.description} /> : null}

      {!gap && options.mode === ETexturePreviewMode.IMAGE && shape && url ? (
        <ImageViewport alt={description.reference} src={url} width={shape.width} height={shape.height} />
      ) : null}

      {!gap && options.mode === ETexturePreviewMode.SURFACE ? (
        <TextureSurface options={options} resetToken={resetToken} />
      ) : null}
    </TexturePreviewFrame>
  );
}
