import { Box, Divider, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useEffect, useMemo, useState } from "react";

import { ArchiveFileDetailRow } from "@/applications/archives-explorer/components/editor/file-details/ArchiveFileDetailRow";
import { formatAudioChannels } from "@/applications/archives-explorer/components/editor/preview/ArchiveAudioPreview/ArchiveAudioPreview.utils";
import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes, TArchiveContent } from "@/core/archive";
import { AssetService } from "@/core/assets/services";
import { AudioDescriptor } from "@/core/bindings/types/xrf-app";
import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { AudioPlayer } from "@/core/ui/media/AudioPlayer";
import { Loadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

/** One sound is previewed at a time, so its url lives under a fixed key and displaces the last one. */
const ARCHIVE_AUDIO_ASSET_KEY: string = "archive-audio";

/** Wide enough for a waveform to be readable, narrow enough that the detail rows stay scannable. */
const ARCHIVE_AUDIO_PREVIEW_WIDTH: number = 640;

/**
 * Plays an archived sound and reports what the engine would read from it.
 */
export function ArchiveAudioPreview(): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const assetService: AssetService = useInjection(AssetService);

  const [url, setUrl] = useState<Nullable<string>>(null);

  const content: Loadable<Nullable<TArchiveContent>> = archivesService.content;
  const audio: Nullable<TArchiveContent & { kind: "audio" }> = content.value?.kind === "audio" ? content.value : null;

  const descriptor: Nullable<AudioDescriptor> = audio?.descriptor ?? null;
  const bytes: Nullable<TArchiveBytes> = audio?.bytes ?? null;
  const blob: Nullable<Blob> = useMemo(() => (bytes ? new Blob([bytes], { type: "audio/ogg" }) : null), [bytes]);

  useEffect(() => {
    setUrl(blob ? assetService.swap(ARCHIVE_AUDIO_ASSET_KEY, blob) : null);
  }, [assetService, blob]);

  if (content.isLoading) {
    return <DelayedProgress />;
  }

  if (content.error) {
    return <ArchivePreviewError error={content.error} onRetry={archivesService.retrySelectedFile} />;
  }

  if (!descriptor || !url) {
    return <EmptyState title={"Preview unavailable"} description={"This sound could not be read."} />;
  }

  return (
    <CenteredColumn
      sx={{
        padding: 3,
        gap: 2.5,
        overflowY: "auto",
        justifyContent: "safe center",
      }}
    >
      <Box sx={{ flexShrink: 0, width: "100%", maxWidth: ARCHIVE_AUDIO_PREVIEW_WIDTH }}>
        <AudioPlayer src={url} bytes={bytes} />
      </Box>

      <Box sx={{ flexShrink: 0, width: "100%", maxWidth: ARCHIVE_AUDIO_PREVIEW_WIDTH }}>
        <Typography variant={"subtitle2"}>Stream</Typography>

        <ArchiveFileDetailRow label={"Channels"} value={formatAudioChannels(descriptor.channels)} />
        <ArchiveFileDetailRow
          label={"Sample rate"}
          value={descriptor.sampleRate ? `${descriptor.sampleRate} Hz` : "-"}
        />

        <Divider sx={{ marginY: 1.5 }} />

        <Typography variant={"subtitle2"}>Engine parameters</Typography>

        {descriptor.parameters ? (
          <>
            <ArchiveFileDetailRow label={"Min distance"} value={`${descriptor.parameters.minDistance} m`} />
            <ArchiveFileDetailRow label={"Max distance"} value={`${descriptor.parameters.maxDistance} m`} />
            <ArchiveFileDetailRow label={"Max AI distance"} value={`${descriptor.parameters.maxAiDistance} m`} />
            <ArchiveFileDetailRow
              label={"Base volume"}
              value={`${descriptor.parameters.baseVolume ?? 0} (${Math.round(
                (descriptor.parameters.baseVolume ?? 0) * 100
              )}%)`}
            />
            <ArchiveFileDetailRow label={"Game type"} value={String(descriptor.parameters.gameType)} mono />
          </>
        ) : (
          <Typography variant={"body2"} sx={{ marginTop: 1, color: "text.secondary" }}>
            This sound carries no X-Ray comment, so the engine would use its built-in source defaults.
          </Typography>
        )}
      </Box>
    </CenteredColumn>
  );
}
