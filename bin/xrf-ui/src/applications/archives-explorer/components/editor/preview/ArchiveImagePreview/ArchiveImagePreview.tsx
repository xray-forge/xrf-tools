import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes, TArchiveContent, useLastContent } from "@/core/archive";
import { describeTextureShape } from "@/core/assets/lib";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { AssetTextureShape } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { AsyncState } from "@/lib/async-state";
import { Nullable } from "@/lib/types/general";

/** One texture is previewed at a time, so its url lives under a fixed key and displaces the last one. */
const ARCHIVE_IMAGE_ASSET_KEY: string = "archive-image";

/**
 * Shows an archived texture the backend decoded into a PNG.
 */
export function ArchiveImagePreview(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

  // The previous texture stays on screen while the next one decodes, rather than the panel blanking between clicks.
  const image: Nullable<TArchiveContent & { kind: "image" }> = useLastContent(
    content.value?.kind === "image" ? content.value : null,
    content.isLoading
  );

  // The shape is the source DDS's rather than the png's, so the caption can name a format and a mip chain the transcode
  // has already thrown away. A header that would not parse leaves nothing to lay the viewport out against.
  const shape: Nullable<AssetTextureShape> = image?.descriptor.shape ?? null;
  const bytes: Nullable<TArchiveBytes> = image?.bytes ?? null;

  const url: Nullable<string> = useAssetUrl(ARCHIVE_IMAGE_ASSET_KEY, bytes, "image/png");

  if (content.error) {
    return <ArchivePreviewError error={content.error} onRetry={archivesService.retrySelectedFile} />;
  }

  if (!shape || !url) {
    return content.isLoading ? (
      <DelayedProgress />
    ) : (
      <EmptyState title={"Preview unavailable"} description={"This texture could not be decoded."} />
    );
  }

  return (
    <Box sx={{ display: "flex", flexDirection: "column", flexGrow: 1, minWidth: 0, minHeight: 0 }}>
      <ImageViewport
        alt={archivesService.selectedFile?.name ?? "Texture"}
        src={url}
        width={shape.width}
        height={shape.height}
      />

      <Box sx={{ flexShrink: 0, paddingX: 1.5, paddingY: 0.5, borderTop: 1, borderColor: "divider" }}>
        <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
          {describeTextureShape(shape)}
        </Typography>
      </Box>
    </Box>
  );
}
