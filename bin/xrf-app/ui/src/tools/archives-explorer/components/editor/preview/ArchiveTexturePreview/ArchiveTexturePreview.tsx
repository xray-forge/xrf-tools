import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { AssetTextureShape } from "@xrf/ipc/types/xrf-app-core";
import { useLastValue } from "@xrf/react/last-value";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ContentState } from "@xrf/ui/layout/content-state";
import { ImageViewport } from "@xrf/ui/media/image-viewport";
import { cn } from "@xrf/ui/theme";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { TArchiveBytes, TArchiveContent } from "@/core/archive/lib";
import { describeTextureShape } from "@/core/assets/lib";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";

/**
 * Shows an archived texture the backend decoded into a PNG.
 */
export function ArchiveTexturePreview({
  "data-testid": dataTestId = "archive-texture-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);
  const content: AsyncState<Nullable<TArchiveContent>> = previewService.content;

  // The previous texture stays on screen while the next one decodes, rather than the panel blanking between clicks.
  const texture: Nullable<TArchiveContent & { kind: "texture" }> = useLastValue(
    content.value?.kind === "texture" ? content.value : null,
    content.isLoading
  );

  // The shape is the source DDS's rather than the png's, so the caption can name a format and a mip chain the transcode
  // has already thrown away. A header that would not parse leaves nothing to lay the viewport out against.
  const shape: Nullable<AssetTextureShape> = texture?.descriptor.shape ?? null;
  const bytes: Nullable<TArchiveBytes> = texture?.bytes ?? null;

  const url: Nullable<string> = useAssetUrl(`${__MODULE_NAME__}/archive-texture`, bytes, "image/png");

  return (
    <ContentState
      data-testid={dataTestId}
      id={id}
      className={className}
      value={shape && url ? { shape, url } : null}
      isLoading={content.isLoading}
      error={content.error}
      errorTitle={"Could not read this file"}
      onRetry={previewService.retry}
      emptyTitle={"Preview unavailable"}
      emptyDescription={"This texture could not be decoded."}
    >
      {(picture: { shape: AssetTextureShape; url: string }) => (
        <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
          <ImageViewport
            alt={previewService.selectedEntry?.name ?? "Texture"}
            src={picture.url}
            width={picture.shape.width}
            height={picture.shape.height}
          />

          <div className={"shrink-0 border-t border-divider px-3 py-1"}>
            <Typography className={"text-text-secondary"} variant={"caption"}>
              {describeTextureShape(picture.shape)}
            </Typography>
          </div>
        </div>
      )}
    </ContentState>
  );
}
