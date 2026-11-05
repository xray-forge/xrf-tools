import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelViewerCaptureDialog } from "@/tools/level-viewer/components/capture/LevelViewerCaptureDialog";
import { LevelViewerCaptureService } from "@/tools/level-viewer/services/level-viewer-capture";

/** The map or cubemap capture's dialog, while it is open. */
export function LevelViewerCaptureDialogHost(): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);

  return captureService.isOpen ? <LevelViewerCaptureDialog /> : <></>;
}
