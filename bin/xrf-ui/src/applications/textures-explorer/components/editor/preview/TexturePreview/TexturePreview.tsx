import { default as ImageIcon } from "@mui/icons-material/Image";
import { default as SurfaceIcon } from "@mui/icons-material/ViewInAr";
import { Button, ToggleButton, ToggleButtonGroup, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useMemo, useState } from "react";

import { TextureSurface } from "@/applications/textures-explorer/components/editor/preview/TextureSurface";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
import { describeTextureShape } from "@/core/assets/lib";
import { AssetService } from "@/core/assets/services";
import { AssetTextureShape, TextureDescription } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { describeTexturePreviewGap, ETexturePreviewMode, ITexturePreviewGap } from "./TexturePreview.utils";
import { TexturePreviewFrame } from "./TexturePreviewFrame";

/** One texture is shown at a time, so its url lives under a fixed key and displaces the last one. */
const TEXTURE_PREVIEW_ASSET_KEY: string = "texture-preview";

/**
 * The selected texture, flat or on a lit body.
 */
export function TexturePreview({
  "data-testid": dataTestId = "texture-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const texturesService: TexturesService = useInjection(TexturesService);
  const assetService: AssetService = useInjection(AssetService);

  const [mode, setMode] = useState<ETexturePreviewMode>(ETexturePreviewMode.IMAGE);
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

  const modeSwitch: ReactElement = (
    <ToggleButtonGroup
      exclusive
      size={"small"}
      value={mode}
      aria-label={"Preview mode"}
      onChange={(_, next: Nullable<ETexturePreviewMode>) => next && setMode(next)}
    >
      <Tooltip title={"The decoded picture, flat"}>
        <ToggleButton value={ETexturePreviewMode.IMAGE} aria-label={"Image"}>
          <ImageIcon fontSize={"small"} />
        </ToggleButton>
      </Tooltip>

      <Tooltip title={"A lit body, shaded through the declared bump pair"}>
        <ToggleButton value={ETexturePreviewMode.SURFACE} aria-label={"Surface"}>
          <SurfaceIcon fontSize={"small"} />
        </ToggleButton>
      </Tooltip>
    </ToggleButtonGroup>
  );

  if (isReading || !description) {
    return (
      <TexturePreviewFrame
        data-testid={dataTestId}
        id={id}
        className={className}
        caption={"Reading…"}
        actions={modeSwitch}
      >
        <DelayedProgress />
      </TexturePreviewFrame>
    );
  }

  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;
  const gap: Nullable<ITexturePreviewGap> = describeTexturePreviewGap(
    mode,
    description.texture !== null,
    Boolean(shape && url)
  );
  // The flat viewport draws its own checkerboard; the lit surface is a transparent canvas and needs the frame's.
  const isCheckered: boolean = !gap && mode === ETexturePreviewMode.SURFACE;

  return (
    <TexturePreviewFrame
      data-testid={dataTestId}
      id={id}
      className={className}
      caption={shape ? describeTextureShape(shape) : description.reference}
      isCheckered={isCheckered}
      actions={modeSwitch}
    >
      {gap ? <EmptyState title={gap.title} description={gap.description} /> : null}

      {!gap && mode === ETexturePreviewMode.IMAGE && shape && url ? (
        <ImageViewport alt={description.reference} src={url} width={shape.width} height={shape.height} />
      ) : null}

      {!gap && mode === ETexturePreviewMode.SURFACE ? <TextureSurface /> : null}
    </TexturePreviewFrame>
  );
}
