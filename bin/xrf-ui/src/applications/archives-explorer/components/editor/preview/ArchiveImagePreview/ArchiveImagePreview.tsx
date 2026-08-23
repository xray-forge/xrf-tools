import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useMemo, useState } from "react";

import { formatMipmapLevels } from "@/applications/archives-explorer/components/editor/preview/ArchiveImagePreview/ArchiveImagePreview.utils";
import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes, TArchiveContent, useLastContent } from "@/core/archive";
import { AssetService } from "@/core/assets/services";
import { AssetTextureShape } from "@/core/bindings/types/xrf-app";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ImageViewport } from "@/core/ui/media/ImageViewport";
import { Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

/** One texture is previewed at a time, so its url lives under a fixed key and displaces the last one. */
const ARCHIVE_IMAGE_ASSET_KEY: string = "archive-image";

/**
 * Shows an archived texture the backend decoded into a PNG.
 */
export function ArchiveImagePreview(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const assetService: AssetService = useInjection(AssetService);

  const [url, setUrl] = useState<Nullable<string>>(null);

  const content: Loadable<Nullable<TArchiveContent>> = archivesService.content;

  // The previous texture stays on screen while the next one decodes, rather than the panel blanking between clicks.
  const image: Nullable<TArchiveContent & { kind: "image" }> = useLastContent(
    content.value?.kind === "image" ? content.value : null,
    content.isLoading
  );

  // The shape is the source DDS's rather than the png's, so the caption can name a format and a mip chain the transcode
  // has already thrown away. A header that would not parse leaves nothing to lay the viewport out against.
  const shape: Nullable<AssetTextureShape> = image?.descriptor.shape ?? null;
  const bytes: Nullable<TArchiveBytes> = image?.bytes ?? null;

  // Blobbed from the view rather than its buffer, so a byte offset cannot silently widen the picture.
  const blob: Nullable<Blob> = useMemo(() => (bytes ? new Blob([bytes], { type: "image/png" }) : null), [bytes]);

  useEffect(() => {
    setUrl(blob ? assetService.swap(ARCHIVE_IMAGE_ASSET_KEY, blob) : null);
  }, [assetService, blob]);

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
          {shape.width} x {shape.height} · {shape.format} · {formatMipmapLevels(shape.mipmapLevels)}
        </Typography>
      </Box>
    </Box>
  );
}
