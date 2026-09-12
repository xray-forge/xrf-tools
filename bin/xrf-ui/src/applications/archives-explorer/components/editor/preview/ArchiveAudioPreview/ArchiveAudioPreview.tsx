import { Box, Divider, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { formatAudioChannels } from "@/applications/archives-explorer/components/editor/preview/ArchiveAudioPreview/ArchiveAudioPreview.utils";
import { ArchivePreviewError } from "@/applications/archives-explorer/components/editor/preview/ArchivePreviewError/ArchivePreviewError";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes, TArchiveContent, useLastContent } from "@/core/archive/lib";
import { useAssetUrl } from "@/core/assets/lib/use-asset-url";
import { AudioDescriptor } from "@/core/bindings/types/xrf-app";
import { EditorPanelProperty } from "@/core/shell/editor/EditorPanel";
import { CenteredColumn } from "@/core/ui/layout/CenteredColumn";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { AudioPlayer } from "@/core/ui/media/AudioPlayer";
import { AsyncState } from "@/lib/async-state";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/**
 * Plays an archived sound and reports what the engine would read from it.
 */
export function ArchiveAudioPreview({
  "data-testid": dataTestId = "archive-audio-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const content: AsyncState<Nullable<TArchiveContent>> = archivesService.content;

  // The previous sound stays on screen while the next one loads, so the transport is never torn down mid-selection.
  const audio: Nullable<TArchiveContent & { kind: "audio" }> = useLastContent(
    content.value?.kind === "audio" ? content.value : null,
    content.isLoading
  );

  const descriptor: Nullable<AudioDescriptor> = audio?.descriptor ?? null;
  const bytes: Nullable<TArchiveBytes> = audio?.bytes ?? null;
  const url: Nullable<string> = useAssetUrl(`${__MODULE_NAME__}/archive-audio`, bytes, "audio/ogg");

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

  if (!descriptor || !url) {
    return content.isLoading ? (
      <DelayedProgress data-testid={dataTestId} id={id} className={className} />
    ) : (
      <EmptyState
        data-testid={dataTestId}
        id={id}
        className={className}
        title={"Preview unavailable"}
        description={"This sound could not be read."}
      />
    );
  }

  return (
    <CenteredColumn
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        padding: 3,
        gap: 2.5,
        overflowY: "auto",
        justifyContent: "safe center",
      }}
    >
      <Box sx={{ flexShrink: 0, width: "100%", maxWidth: 640 }}>
        <AudioPlayer src={url} bytes={bytes} />
      </Box>

      <Box sx={{ flexShrink: 0, width: "100%", maxWidth: 640 }}>
        <Typography variant={"subtitle2"}>Stream</Typography>

        <EditorPanelProperty label={"Channels"} value={formatAudioChannels(descriptor.channels)} />
        <EditorPanelProperty
          label={"Sample rate"}
          value={descriptor.sampleRate ? `${descriptor.sampleRate} Hz` : "-"}
        />

        <Divider sx={{ marginY: 1.5 }} />

        <Typography variant={"subtitle2"}>Engine parameters</Typography>

        {descriptor.parameters ? (
          <>
            <EditorPanelProperty label={"Min distance"} value={`${descriptor.parameters.minDistance} m`} />
            <EditorPanelProperty label={"Max distance"} value={`${descriptor.parameters.maxDistance} m`} />
            <EditorPanelProperty label={"Max AI distance"} value={`${descriptor.parameters.maxAiDistance} m`} />
            <EditorPanelProperty
              label={"Base volume"}
              value={`${descriptor.parameters.baseVolume ?? 0} (${Math.round(
                (descriptor.parameters.baseVolume ?? 0) * 100
              )}%)`}
            />
            <EditorPanelProperty label={"Game type"} value={String(descriptor.parameters.gameType)} isMonospace />
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
