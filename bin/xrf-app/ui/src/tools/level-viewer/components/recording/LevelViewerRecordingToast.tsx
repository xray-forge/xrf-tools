import { useInjection } from "@wirestate/react";
import { RecordingOutcome } from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VideocamIcon } from "@xrf/ui/icons/videocam";
import { FileToast } from "@xrf/ui/toast";
import { ReactElement, useCallback } from "react";

import { LevelViewerRecordingReadout } from "@/tools/level-viewer/lib/recording/level-viewer-recording-readout";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";

/**
 * The recording just written, with its readout and the file actions a screenshot's toast offers.
 */
export function LevelViewerRecordingToast({
  "data-testid": dataTestId = "level-viewer-recording-toast",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const recordingService: LevelViewerRecordingService = useInjection(LevelViewerRecordingService);
  const outcome: Nullable<RecordingOutcome> = recordingService.outcome;

  const onReveal = useCallback(() => void recordingService.reveal(), [recordingService]);

  const onCopyPath = useCallback(() => void recordingService.copyPath(), [recordingService]);

  const onDelete = useCallback(() => void recordingService.remove(), [recordingService]);

  return (
    <FileToast
      data-testid={dataTestId}
      id={id}
      className={className}
      file={outcome?.file ?? null}
      title={"Recording saved"}
      name={outcome ? LevelViewerRecordingReadout.getFileName(outcome.file) : ""}
      details={outcome ? LevelViewerRecordingReadout.describeOutcome(outcome) : ""}
      isWarned={outcome ? LevelViewerRecordingReadout.hasDrops(outcome) : false}
      preview={<VideocamIcon className={"text-text-secondary"} />}
      onReveal={onReveal}
      onCopyPath={onCopyPath}
      onDelete={onDelete}
    />
  );
}
