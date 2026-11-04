import { useInjection } from "@wirestate/react";
import { formatDuration } from "@xrf/format";
import { RecordingProgress } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { EditorViewToggle } from "@xrf/ui/editor/view-toggle";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VideocamIcon } from "@xrf/ui/icons/videocam";
import { ReactElement } from "react";

import { describeDrops, LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";

/**
 * @param progress - Where the recording stands, null before its first word.
 * @returns How long it has run and what it dropped, for its tooltip.
 */
function describeProgress(progress: Nullable<RecordingProgress>): string {
  if (!progress) {
    return "Recording…";
  }

  return `Recording ${formatDuration((progress.elapsed ?? 0) * 1000)}, ${describeDrops(progress)}. Click to stop.`;
}

/**
 * Records the viewport with what its camera hears into the recordings folder on a click, and stops on the next; lit
 * while it records, its tooltip the time and the drops. Off, saying why, where this machine cannot record.
 */
export function LevelViewerRecordAction({
  "data-testid": dataTestId = "level-viewer-record-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const recordingService: LevelViewerRecordingService = useInjection(LevelViewerRecordingService);
  const isRecording: boolean = recordingService.isRecording;

  return (
    <EditorViewToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Record"}
      description={
        isRecording ? describeProgress(recordingService.progress) : "Record the viewport with its sound to a video"
      }
      icon={<VideocamIcon />}
      isOn={isRecording}
      isDisabled={!isRecording && !recordingService.canStart}
      unavailableTitle={recordingService.availability?.reason ?? undefined}
      onToggle={() => void recordingService.toggle()}
    />
  );
}
