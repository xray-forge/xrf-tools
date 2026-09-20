import { default as VideocamIcon } from "@mui/icons-material/Videocam";
import { Button, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import {
  DEFAULT_LEVEL_CAMERA_OPTIONS,
  ILevelCameraOptions,
  LEVEL_CAMERA_LIMITS,
} from "@/core/level/lib/camera/level-camera-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDegrees } from "@/lib/format/angle";
import { formatNumber } from "@/lib/format/number";

interface ILevelCameraActionProps extends BaseComponentProps {
  camera: ILevelCameraOptions;
  onChange: (camera: ILevelCameraOptions) => void;
}

/**
 * What the camera sees and how it answers input, in the toolbar beside the rest of what a person sets.
 */
export function LevelCameraAction({
  "data-testid": dataTestId = "level-camera-action",
  id,
  className,
  camera,
  onChange,
}: ILevelCameraActionProps): ReactElement {
  const set = useCallback((part: Partial<ILevelCameraOptions>) => onChange({ ...camera, ...part }), [camera, onChange]);

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
          onChange={(fieldOfView) => set({ fieldOfView })}
        />

        <RenderValueSlider
          label={"Speed"}
          value={camera.speed}
          {...LEVEL_CAMERA_LIMITS.speed}
          format={(value: number) => `${value} m/s`}
          onChange={(speed) => set({ speed })}
        />

        <RenderValueSlider
          label={"Boost"}
          value={camera.boost}
          {...LEVEL_CAMERA_LIMITS.boost}
          format={(value: number) => `${value}×`}
          onChange={(boost) => set({ boost })}
        />

        <RenderValueSlider
          label={"Sensitivity"}
          value={camera.sensitivity}
          {...LEVEL_CAMERA_LIMITS.sensitivity}
          format={(value: number) => formatNumber(value, 4)}
          onChange={(sensitivity) => set({ sensitivity })}
        />

        <Button size={"small"} onClick={() => onChange(DEFAULT_LEVEL_CAMERA_OPTIONS)}>
          Back to the default camera
        </Button>
      </div>
    </EditorPopoverAction>
  );
}
