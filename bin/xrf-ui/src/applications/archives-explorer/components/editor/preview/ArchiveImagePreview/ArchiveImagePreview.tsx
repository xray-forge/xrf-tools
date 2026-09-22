import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes, TArchiveContent, useLastContent } from "@/core/archive/lib";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { ImageShape } from "@/core/ipc/types/xrf-texture";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { AsyncState } from "@/lib/async-state";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Shows a picture the webview draws as it stands, which the engine does not load.
 */
export function ArchiveImagePreview({
  "data-testid": dataTestId = "archive-image-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

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
        onRetry={archivesService.retrySelectedFile}
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
        alt={archivesService.selectedEntry?.name ?? "Picture"}
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
