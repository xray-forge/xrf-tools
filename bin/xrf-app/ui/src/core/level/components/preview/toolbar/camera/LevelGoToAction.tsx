import { default as MyLocationIcon } from "@mui/icons-material/MyLocation";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { LevelGoToForm } from "@/core/level/components/preview/toolbar/camera/LevelGoToForm";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { LevelLoadService } from "@/core/level/services/level-load/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render/level-render.service";
import { LevelViewportService } from "@/core/level/services/level-viewport/level-viewport.service";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Stands the camera at a place typed in, in the coordinates the readout shows, or pasted from it.
 */
export function LevelGoToAction({
  "data-testid": dataTestId = "level-go-to-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  const isDisabled: boolean = loadService.level.value === null;

  // Where the camera is as the popover opens, which the fields start at; null before the viewport has drawn.
  const readCamera = useCallback((): Nullable<ILevelCamera> => viewportService.camera, [viewportService]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Go to"}
      description={"Go to coordinates"}
      icon={<MyLocationIcon />}
      isDisabled={isDisabled}
    >
      <LevelGoToForm readCamera={readCamera} onGoTo={renderService.goTo} />
    </EditorPopoverAction>
  );
}
