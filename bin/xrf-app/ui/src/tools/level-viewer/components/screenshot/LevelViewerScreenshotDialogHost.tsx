import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelViewerScreenshotDialog } from "@/tools/level-viewer/components/screenshot/LevelViewerScreenshotDialog";
import { ILevelViewerScreenshot } from "@/tools/level-viewer/lib/screenshot/level-viewer-screenshot";
import { LevelViewerScreenshotService } from "@/tools/level-viewer/services/level-viewer-screenshot";

/** The last screenshot's dialog, while it is shown; what fails with one is noticed. */
export function LevelViewerScreenshotDialogHost(): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);
  const shot: Nullable<ILevelViewerScreenshot> = screenshotService.shot.value;

  return screenshotService.isShown && shot ? <LevelViewerScreenshotDialog shot={shot} /> : <></>;
}
