import { default as VideocamIcon } from "@mui/icons-material/Videocam";
import { Button, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatDegrees } from "@xrf/format";
import { ReactElement } from "react";

import {
  DEFAULT_LEVEL_CAMERA_OPTIONS,
  formatCameraSensitivity,
  ILevelCameraOptions,
  LEVEL_CAMERA_LIMITS,
} from "@/core/level/lib/camera/level-camera-options";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { usePartialChange } from "@/lib/react/use-partial-change";

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

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Camera"}
      description={`Camera: ${formatDegrees(camera.fieldOfView)}, ${camera.speed} m/s`}
      icon={<VideocamIcon />}
    >
      <div className={"flex w-56 flex-col gap-2 px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Camera
        </Typography>

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

        <Button size={"small"} onClick={() => viewService.setCameraOptions(DEFAULT_LEVEL_CAMERA_OPTIONS)}>
          Reset the camera
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
