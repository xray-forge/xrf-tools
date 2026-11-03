import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CenteredColumn } from "@xrf/ui/layout/centered-column";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { Alert } from "@xrf/ui/material/alert";
import { Divider } from "@xrf/ui/material/divider";
import { Typography } from "@xrf/ui/material/typography";
import { AudioPlayer } from "@xrf/ui/media/audio-player";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { TArchiveContent, useLastContent } from "@/core/archive/lib";
import { IAudioPreview, useAudioPreview } from "@/core/audio/lib/use-audio-preview";
import { AudioSettingsService } from "@/core/audio/services/audio-settings.service";
import { ArchivePreviewError } from "@/tools/archives-explorer/components/editor/preview/ArchivePreviewError/ArchivePreviewError";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";

import { formatAudioChannels } from "./ArchiveAudioPreview.utils";

/**
 * Plays an archived sound through the application's audio, on the Previews volume, and reports what the engine would
 * read from it.
 */
export function ArchiveAudioPreview({
  "data-testid": dataTestId = "archive-audio-preview",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);
  const audioSettingsService: AudioSettingsService = useInjection(AudioSettingsService);
  const content: AsyncState<Nullable<TArchiveContent>> = previewService.content;

  // The previous sound stays on screen while the next one loads, so the player is never torn down mid-selection.
  const audio: Nullable<TArchiveContent & { kind: "audio" }> = useLastContent(
    content.value?.kind === "audio" ? content.value : null,
    content.isLoading
  );

  const descriptor: Nullable<AudioDescriptor> = audio?.descriptor ?? null;
  const preview: IAudioPreview = useAudioPreview(
    audio
      ? {
          key: `${audio.sessionId}/${audio.name}`,
          open: () => archivesCommands.openAudioPreview(audio.sessionId, audio.name),
        }
      : null
  );

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

  if (!descriptor) {
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
      className={cn("justify-center-safe gap-5 overflow-y-auto p-6", className)}
    >
      <div className={"w-full max-w-160 shrink-0"}>
        {preview.error ? <Alert severity={"error"}>{preview.error}</Alert> : null}

        {preview.transport ? (
          <AudioPlayer
            transport={preview.transport}
            volume={audioSettingsService.previewsVolume}
            onVolumeChange={audioSettingsService.setPreviewsVolume}
          />
        ) : preview.error ? null : (
          <DelayedProgress />
        )}
      </div>

      <div className={"w-full max-w-160 shrink-0"}>
        <Typography variant={"subtitle2"}>Stream</Typography>

        <EditorPanelProperty label={"Channels"} value={formatAudioChannels(descriptor.channels)} />
        <EditorPanelProperty
          label={"Sample rate"}
          value={descriptor.sampleRate ? `${descriptor.sampleRate} Hz` : "-"}
        />

        <Divider className={"my-3"} />

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
          <Typography className={"mt-2 text-text-secondary"} variant={"body2"}>
            This sound carries no X-Ray comment, so the engine would use its built-in source defaults.
          </Typography>
        )}
      </div>
    </CenteredColumn>
  );
}
