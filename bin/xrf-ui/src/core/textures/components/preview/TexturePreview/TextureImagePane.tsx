import { Dispatch, ReactElement, SetStateAction } from "react";

import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { AssetTextureShape } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Loadable } from "@/lib/loadable";
import { IPanZoomState } from "@/lib/media/pan-zoom";
import { Nullable } from "@/lib/types/general";

import { TexturePreviewFrame } from "./TexturePreviewFrame";

interface ITextureImagePaneProps extends BaseComponentProps {
  /** What this picture is, on the bar beneath it. */
  caption: string;
  alt: string;
  /** Slot the decoded bytes are held under, unique to this pane. */
  assetKey: string;
  /** The picture, as png bytes. */
  preview: Loadable<Nullable<ArrayBuffer>>;
  /** How large it is, which the viewport lays itself out against. */
  shape: Nullable<AssetTextureShape>;
  state?: IPanZoomState;
  onStateChange?: Dispatch<SetStateAction<IPanZoomState>>;
  hasControls?: boolean;
}

/**
 * One captioned picture in the preview area.
 *
 * Made a component of its own for the pair: two of these showing the same part of two encodes is what a comparison
 * is, and they can only move together if neither of them owns the pan.
 */
export function TextureImagePane({
  "data-testid": dataTestId = "texture-image-pane",
  id,
  className,
  caption,
  alt,
  assetKey,
  preview,
  shape,
  state,
  onStateChange,
  hasControls,
}: ITextureImagePaneProps): ReactElement {
  const url: Nullable<string> = useAssetUrl(assetKey, preview.value);

  return (
    <TexturePreviewFrame data-testid={dataTestId} id={id} className={className} caption={caption}>
      {preview.error ? <EmptyState title={"Could not show this encoding"} description={preview.error.message} /> : null}

      {!preview.error && (preview.isLoading || !url || !shape) ? <DelayedProgress /> : null}

      {!preview.error && url && shape ? (
        <ImageViewport
          alt={alt}
          src={url}
          width={shape.width}
          height={shape.height}
          state={state}
          onStateChange={onStateChange}
          hasControls={hasControls}
        />
      ) : null}
    </TexturePreviewFrame>
  );
}
