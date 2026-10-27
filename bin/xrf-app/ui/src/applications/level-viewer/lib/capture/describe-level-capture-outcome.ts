import { CaptureOutcome } from "@xrf/ipc/types/xrf-app-render";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";

import { ELevelCaptureMode, LEVEL_CAPTURE_NAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import { toLevelCubeFaceSize } from "@/applications/level-viewer/lib/cube/level-cube-capture";
import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a capture ends.
 *
 * @param mode - What was captured.
 * @param path - The PNG it was written to.
 * @param outcome - How the run ended.
 * @returns The notice.
 */
export function describeLevelCaptureOutcome(
  mode: ELevelCaptureMode,
  path: string,
  outcome: IJobOutcome<CaptureOutcome>
): IJobNotice {
  const { result, error } = outcome;
  const name: string = LEVEL_CAPTURE_NAMES[mode];

  if (error) {
    return describeJobFailure(`Could not capture the ${name}`, path, error);
  }

  if (result?.outcome === EJobOutcome.CANCELLED) {
    return {
      details: `Nothing was written: the ${name} is written only once every ${
        mode === ELevelCaptureMode.MAP ? "tile" : "face"
      } is back.`,
      severity: ENotificationSeverity.INFO,
      title: `Capturing the ${name} was stopped`,
    };
  }

  return {
    details: path,
    severity: ENotificationSeverity.SUCCESS,
    title:
      mode === ELevelCaptureMode.MAP
        ? `Captured a ${result?.width ?? 0} × ${result?.height ?? 0} level map`
        : `Captured a cubemap of ${toLevelCubeFaceSize(result?.width ?? 0)} pixel faces`,
  };
}
