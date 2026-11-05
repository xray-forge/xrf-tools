import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { useShellStatus } from "@/core/shell/contributions";
import { LevelViewerRecordingReadout } from "@/tools/level-viewer/lib/recording/level-viewer-recording-readout";
import { LevelViewerRecordingService } from "@/tools/level-viewer/services/level-viewer-recording";

/** The recording running, in the status bar: how long it has run and what it dropped. */
export function LevelViewerRecordingStatus(): ReactElement {
  const recordingService: LevelViewerRecordingService = useInjection(LevelViewerRecordingService);
  const isRecording: boolean = recordingService.isRecording;
  const progress: string = LevelViewerRecordingReadout.describeProgress(recordingService.progress);

  useShellStatus(isRecording ? [progress] : []);

  return <></>;
}
