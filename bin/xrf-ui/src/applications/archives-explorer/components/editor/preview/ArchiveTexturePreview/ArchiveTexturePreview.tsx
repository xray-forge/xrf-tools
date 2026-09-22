import { Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes, TArchiveContent, useLastContent } from "@/core/archive/lib";
import { describeTextureShape } from "@/core/assets/lib";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { AssetTextureShape } from "@/core/ipc/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { AsyncState } from "@/lib/async-state";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Shows an archived texture the backend decoded into a PNG.
 */
export function ArchiveTexturePreview({
  "data-testid": dataTestId = "archive-texture-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

  // The previous texture stays on screen while the next one decodes, rather than the panel blanking between clicks.
  const texture: Nullable<TArchiveContent & { kind: "texture" }> = useLastContent(
    content.value?.kind === "texture" ? content.value : null,
    content.isLoading
  );

  // The shape is the source DDS's rather than the png's, so the caption can name a format and a mip chain the transcode
  // has already thrown away. A header that would not parse leaves nothing to lay the viewport out against.
  const shape: Nullable<AssetTextureShape> = texture?.descriptor.shape ?? null;
  const bytes: Nullable<TArchiveBytes> = texture?.bytes ?? null;

  const url: Nullable<string> = useAssetUrl(`${__MODULE_NAME__}/archive-texture`, bytes, "image/png");

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
        description={"This texture could not be decoded."}
      />
    );
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 grow flex-col", className)}>
      <ImageViewport
        alt={archivesService.selectedEntry?.name ?? "Texture"}
        src={url}
        width={shape.width}
        height={shape.height}
      />

      <div className={"shrink-0 border-t border-divider px-3 py-1"}>
        <Typography className={"text-text-secondary"} variant={"caption"}>
          {describeTextureShape(shape)}
        </Typography>
      </div>
    </div>
  );
}
