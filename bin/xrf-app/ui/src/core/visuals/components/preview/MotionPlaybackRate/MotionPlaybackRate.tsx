import { EditorPopoverAction } from "@xrf/ui/editor/popover-action";
import { EditorPopoverContent } from "@xrf/ui/editor/popover-content";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { SpeedIcon } from "@xrf/ui/icons/speed";
import { ReactElement, useCallback } from "react";

import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { MOTION_MAX_FPS, MOTION_MIN_FPS, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";

interface IMotionPlaybackRateProps extends BaseComponentProps {
  fps: number;
  onChange: (fps: number) => void;
}

/**
 * Playback-rate picker shared by single motions and sequence tracks, set back from its heading to the rate a motion
 * samples at.
 */
export function MotionPlaybackRate({
  "data-testid": dataTestId = "motion-playback-rate",
  id,
  className,
  fps,
  onChange,
}: IMotionPlaybackRateProps): ReactElement {
  const onReset = useCallback(() => onChange(MOTION_SAMPLE_FPS), [onChange]);

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
      <EditorPopoverContent
        label={"Playback rate"}
        resetLabel={`Reset to ${MOTION_SAMPLE_FPS} fps, the rate motions sample at`}
        onReset={fps === MOTION_SAMPLE_FPS ? undefined : onReset}
      >
        <RenderValueSlider
          label={"Frames a second"}
          value={fps}
          min={MOTION_MIN_FPS}
          max={MOTION_MAX_FPS}
          step={1}
          format={(value: number) => `${value} fps`}
          onChange={onChange}
        />
      </EditorPopoverContent>
    </EditorPopoverAction>
  );
}
