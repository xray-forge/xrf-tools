import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LEVEL_CAPTURE_NAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import { LevelViewerCaptureService } from "@/applications/level-viewer/services/level-viewer-capture";
import { cn } from "@/core/theme/class-name";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What stands in the viewport while a level map or cubemap is captured: the viewport draws the capture's tiles or
 * faces rather than its own frames, so it is covered, opaque and taking the pointer, until the capture ends.
 */
export function LevelViewerCaptureCover({
  "data-testid": dataTestId = "level-viewer-capture-cover",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const captureService: LevelViewerCaptureService = useInjection(LevelViewerCaptureService);

  if (!captureService.isCapturing) {
    return <></>;
  }

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("absolute inset-0 z-20 flex items-center justify-center bg-viewport-backdrop", className)}
    >
      <DelayedProgress isOnViewport={true} label={`Capturing the ${LEVEL_CAPTURE_NAMES[captureService.mode]}…`} />
    </div>
  );
}
