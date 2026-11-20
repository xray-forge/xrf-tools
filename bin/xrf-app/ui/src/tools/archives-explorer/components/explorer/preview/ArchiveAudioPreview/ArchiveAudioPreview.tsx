import { useInjection } from "@wirestate/react";
import { AsyncState } from "@xrf/async";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { useLastValue } from "@xrf/react/last-value";
import { Nullable } from "@xrf/types";
import { EditorPanelProperty } from "@xrf/ui/editor/panel";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Alert } from "@xrf/ui/feedback";
import { CenteredColumn } from "@xrf/ui/layout/centered-column";
import { ContentState } from "@xrf/ui/layout/content-state";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { Divider } from "@xrf/ui/layout/stack";
import { AudioPlayer } from "@xrf/ui/media/audio-player";
import { cn } from "@xrf/ui/theme";
import { Typography } from "@xrf/ui/typography";
import { ReactElement } from "react";

import { TArchiveContent } from "@/core/archive/lib/selection";
import { IAudioPreview, useAudioPreview } from "@/core/audio/lib/use-audio-preview";
import { AudioSettingsService } from "@/core/audio/services/audio-settings.service";
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
  const audio: Nullable<TArchiveContent & { kind: "audio" }> = useLastValue(
    content.value?.kind === "audio" ? content.value : null,
    content.isLoading
  );

  const descriptor: Nullable<AudioDescriptor> = audio?.descriptor ?? null;
  const preview: IAudioPreview = useAudioPreview(
    audio
      ? {
          key: `${audio.sessionId}/${audio.name}`,
          open: () => previewService.openAudio(audio),
        }
      : null
  );

  return (
    <ContentState
      data-testid={dataTestId}
      id={id}
      className={className}
      value={descriptor}
      isLoading={content.isLoading}
      error={content.error}
      errorTitle={"Could not read this file"}
      onRetry={previewService.retry}
      emptyTitle={"Preview unavailable"}
      emptyDescription={"This sound could not be read."}
    >
      {(sound: AudioDescriptor) => (
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

            <EditorPanelProperty label={"Channels"} value={formatAudioChannels(sound.channels)} />
            <EditorPanelProperty label={"Sample rate"} value={sound.sampleRate ? `${sound.sampleRate} Hz` : "-"} />

            <Divider className={"my-3"} />

            <Typography variant={"subtitle2"}>Engine parameters</Typography>

            {sound.parameters ? (
              <>
                <EditorPanelProperty label={"Min distance"} value={`${sound.parameters.minDistance} m`} />
                <EditorPanelProperty label={"Max distance"} value={`${sound.parameters.maxDistance} m`} />
                <EditorPanelProperty label={"Max AI distance"} value={`${sound.parameters.maxAiDistance} m`} />
                <EditorPanelProperty
                  label={"Base volume"}
                  value={`${sound.parameters.baseVolume ?? 0} (${Math.round((sound.parameters.baseVolume ?? 0) * 100)}%)`}
                />
                <EditorPanelProperty label={"Game type"} value={String(sound.parameters.gameType)} isMonospace />
              </>
            ) : (
              <Typography className={"mt-2 text-text-secondary"} variant={"body2"}>
                This sound carries no X-Ray comment, so the engine would use its built-in source defaults.
              </Typography>
            )}
          </div>
        </CenteredColumn>
      )}
    </ContentState>
  );
}
