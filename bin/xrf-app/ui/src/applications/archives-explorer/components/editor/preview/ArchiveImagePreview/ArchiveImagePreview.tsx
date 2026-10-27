import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { ImageShape } from "@xrf/ipc/types/xrf-texture";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { Typography } from "@xrf/ui/material/typography";
import { ImageViewport } from "@xrf/ui/media/image-viewport";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError/ArchivePreviewError";
import { ArchivePreviewService } from "@/applications/archives-explorer/services/archive-preview";
import { TArchiveBytes, TArchiveContent, useLastContent } from "@/core/archive/lib";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";

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
  const image: Nullable<TArchiveContent & { kind: "image" }> = useLastContent(
    content.value?.kind === "image" ? content.value : null,
    content.isLoading
  );

  const shape: Nullable<ImageShape> = image?.descriptor.shape ?? null;
  const bytes: Nullable<TArchiveBytes> = image?.bytes ?? null;

  // The type comes from the backend rather than a table here, so one spelling decides how the bytes are handed over.
  const url: Nullable<string> = useAssetUrl(`${__MODULE_NAME__}/archive-image`, bytes, image?.descriptor.mediaType);

  if (content.error) {
    return (
      <ArchivePreviewError
        data-testid={dataTestId}
        id={id}
        className={className}
        error={content.error}
        onRetry={previewService.retry}
      />
    );
  }

  if (!shape || !url) {
    return content.isLoading ? (
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    ) : (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"Preview unavailable"}
        description={"This picture carries no header that could be read."}
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
      <ImageViewport
        alt={previewService.selectedEntry?.name ?? "Picture"}
        src={url}
        width={shape.width}
        height={shape.height}
      />

      <div className={"shrink-0 border-t border-divider px-3 py-1"}>
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {`${shape.width} x ${shape.height} · ${shape.format} · image`}
        </Typography>
      </div>
    </div>
  );
}
