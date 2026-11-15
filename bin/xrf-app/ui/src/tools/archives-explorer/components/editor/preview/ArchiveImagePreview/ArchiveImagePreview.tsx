import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { ImageShape } from "@xrf/ipc/types/xrf-texture";
import { useLastValue } from "@xrf/react/last-value";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ContentState } from "@xrf/ui/layout/content-state";
import { ImageViewport } from "@xrf/ui/media/image-viewport";
import { cn } from "@xrf/ui/theme";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { TArchiveBytes, TArchiveContent } from "@/core/archive/lib/selection";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";

/**
 * Shows a picture the webview draws as it stands, which the engine does not load.
 */
export function ArchiveImagePreview({
  "data-testid": dataTestId = "archive-image-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);
  const content: AsyncState<Nullable<TArchiveContent>> = previewService.content;

  // The previous picture stays on screen while the next one loads, the same courtesy the other previews extend.
  const image: Nullable<TArchiveContent & { kind: "image" }> = useLastValue(
    content.value?.kind === "image" ? content.value : null,
    content.isLoading
  );

  const shape: Nullable<ImageShape> = image?.descriptor.shape ?? null;
  const bytes: Nullable<TArchiveBytes> = image?.bytes ?? null;

  // The type comes from the backend rather than a table here, so one spelling decides how the bytes are handed over.
  const url: Nullable<string> = useAssetUrl(`${__MODULE_NAME__}/archive-image`, bytes, image?.descriptor.mediaType);

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
      emptyDescription={"This picture carries no header that could be read."}
    >
      {(picture: { shape: ImageShape; url: string }) => (
        <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
          <ImageViewport
            alt={previewService.selectedEntry?.name ?? "Picture"}
            src={picture.url}
            width={picture.shape.width}
            height={picture.shape.height}
          />

          <div className={"shrink-0 border-t border-divider px-3 py-1"}>
            <Typography className={"text-text-secondary"} variant={"caption"}>
              {`${picture.shape.width} x ${picture.shape.height} · ${picture.shape.format} · image`}
            </Typography>
          </div>
        </div>
      )}
    </ContentState>
  );
}
