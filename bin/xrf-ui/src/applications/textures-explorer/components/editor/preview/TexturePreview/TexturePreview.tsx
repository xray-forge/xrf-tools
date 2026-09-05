import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useMemo, useState } from "react";

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

/**
 * The selected texture as a flat picture.
 */
export function TexturePreview({
  "data-testid": dataTestId = "texture-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
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

  if (isReading) {
    return (
      <TexturePreviewFrame data-testid={dataTestId} id={id} className={className} caption={"Reading…"}>
        <DelayedProgress />
      </TexturePreviewFrame>
    );
  }

  if (!description) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={"No texture open"}
        description={"Pick a texture in the tree to see it, and what its descriptor declares."}
      />
    );
  }

  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;

  if (!shape || !url) {
    return (
      <EmptyState
        data-testid={dataTestId}
        title={description.texture ? "Preview unavailable" : "Descriptor only"}
        description={
          description.texture
            ? "This texture is a layout the backend cannot decode. Its descriptor is still read, beside this."
            : "No texture sits beside this descriptor. The engine reads the descriptor either way, and the panels " +
              "show what it declares."
        }
      />
    );
  }

  return (
    <TexturePreviewFrame
      data-testid={dataTestId}
      id={id}
      className={className}
      caption={describeTextureShape(shape)}
      isCheckered={false}
    >
      <ImageViewport alt={description.reference} src={url} width={shape.width} height={shape.height} />
    </TexturePreviewFrame>
  );
}
