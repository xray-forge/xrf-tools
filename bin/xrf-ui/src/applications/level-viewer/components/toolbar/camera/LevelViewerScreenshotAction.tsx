import { default as PhotoCameraIcon } from "@mui/icons-material/PhotoCameraOutlined";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { LevelViewerScreenshotService } from "@/applications/level-viewer/services/screenshot";
import { EditorIconAction } from "@/core/shell/editor/EditorIconAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** Takes a screenshot, as `F9` does, lit while one is being taken. */
export function LevelViewerScreenshotAction({
  "data-testid": dataTestId = "level-viewer-screenshot-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const screenshotService: LevelViewerScreenshotService = useInjection(LevelViewerScreenshotService);

  const onClick = useCallback(() => void screenshotService.take(), [screenshotService]);

  return (
    <EditorIconAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Screenshot"}
      description={
        screenshotService.isTaking
          ? "Taking a screenshot…"
          : "Take a screenshot (F9): the frame without its overlays, saved to the screenshots folder"
      }
      icon={<PhotoCameraIcon />}
      isActive={screenshotService.isTaking}
      isDisabled={!screenshotService.canTake && !screenshotService.isTaking}
      onClick={onClick}
    />
  );
}
