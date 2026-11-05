import { useInjection } from "@wirestate/react";
import { formatDegrees } from "@xrf/format";
import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { VideocamIcon } from "@xrf/ui/icons/videocam";
import { ReactElement, useCallback } from "react";

import {
  DEFAULT_LEVEL_CAMERA_OPTIONS,
  formatCameraSensitivity,
  ILevelCameraOptions,
  isDefaultLevelCamera,
  LEVEL_CAMERA_LIMITS,
} from "@/core/level/lib/camera/level-camera-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { usePartialChange } from "@/lib/react/use-partial-change";

/** What the camera's reset says it does: it sees and moves as it did before anyone touched it. */
const LEVEL_CAMERA_RESET_LABEL: string = "Reset to the defaults";

/**
 * What the camera sees and how it answers input, in the toolbar beside the rest of what a person sets.
 */
export function LevelCameraAction({
  "data-testid": dataTestId = "level-camera-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);
  const camera: ILevelCameraOptions = viewService.cameraOptions;

  const onSet = usePartialChange(camera, viewService.setCameraOptions);

  const onReset = useCallback(() => viewService.setCameraOptions(DEFAULT_LEVEL_CAMERA_OPTIONS), [viewService]);

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Camera"}
      description={`${formatDegrees(camera.fieldOfView)} field of view, ${camera.speed} m/s`}
      icon={<VideocamIcon />}
    >
      <EditorPopoverContent
        label={"Camera"}
        resetLabel={LEVEL_CAMERA_RESET_LABEL}
        onReset={isDefaultLevelCamera(camera) ? undefined : onReset}
      >
        <RenderValueSlider
          label={"Field of view"}
          value={camera.fieldOfView}
          {...LEVEL_CAMERA_LIMITS.fieldOfView}
          format={formatDegrees}
          onChange={(fieldOfView) => onSet({ fieldOfView })}
        />

        <RenderValueSlider
          label={"Speed"}
          value={camera.speed}
          {...LEVEL_CAMERA_LIMITS.speed}
          format={(value: number) => `${value} m/s`}
          onChange={(speed) => onSet({ speed })}
        />

        <RenderValueSlider
          label={"Boost"}
          value={camera.boost}
          {...LEVEL_CAMERA_LIMITS.boost}
          format={(value: number) => `${value}×`}
          onChange={(boost) => onSet({ boost })}
        />

        <RenderValueSlider
          label={"Sensitivity"}
          value={camera.sensitivity}
          {...LEVEL_CAMERA_LIMITS.sensitivity}
          format={formatCameraSensitivity}
          onChange={(sensitivity) => onSet({ sensitivity })}
        />
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
