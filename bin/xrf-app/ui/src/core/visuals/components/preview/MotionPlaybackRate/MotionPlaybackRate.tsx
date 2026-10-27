import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { SpeedIcon } from "@xrf/ui/icons/speed";
import { Slider } from "@xrf/ui/material/slider";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback } from "react";

import { MOTION_MAX_FPS, MOTION_MIN_FPS, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";

interface IMotionPlaybackRateProps extends BaseComponentProps {
  fps: number;
  onChange: (fps: number) => void;
}

/**
 * Playback-rate picker shared by single motions and sequence tracks.
 */
export function MotionPlaybackRate({
  "data-testid": dataTestId = "motion-playback-rate",
  id,
  className,
  fps,
  onChange,
}: IMotionPlaybackRateProps): ReactElement {
  const onChangeFps = useCallback(
    (_: Event, value: number | Array<number>) => {
      if (typeof value === "number") {
        onChange(value);
      }
    },
    [onChange]
  );

  return (
    <EditorPopoverAction
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Playback rate"}
      description={`Playback rate: ${fps} fps`}
      icon={<SpeedIcon />}
      isActive={fps !== MOTION_SAMPLE_FPS}
      placement={"top"}
    >
      <div className={"w-toolbar-slider px-4 py-2"}>
        <Typography className={"text-text-secondary"} variant={"overline"}>
          Playback rate
        </Typography>

        <Slider
          aria-label={"Frames a second"}
          size={"small"}
          min={MOTION_MIN_FPS}
          max={MOTION_MAX_FPS}
          value={fps}
          valueLabelDisplay={"auto"}
          valueLabelFormat={(value: number) => `${value} fps`}
          marks={[{ value: MOTION_SAMPLE_FPS, label: String(MOTION_SAMPLE_FPS) }]}
          onChange={onChangeFps}
        />
      </div>
    </EditorPopoverAction>
  );
}
