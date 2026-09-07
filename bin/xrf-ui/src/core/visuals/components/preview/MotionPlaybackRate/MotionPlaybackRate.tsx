import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { Box, Slider, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { EditorPopoverAction } from "@/core/shell/editor/EditorPopoverAction";
import { LAYOUT } from "@/core/theme/tokens";
import { MOTION_MAX_FPS, MOTION_MIN_FPS, MOTION_SAMPLE_FPS } from "@/core/visuals/lib/visual-motion";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
      isHighlighted={fps !== MOTION_SAMPLE_FPS}
      placement={"top"}
    >
      <Box sx={{ paddingX: 2, paddingY: 1, width: LAYOUT.toolbarSliderWidth }}>
        <Typography variant={"overline"} sx={{ color: "text.secondary" }}>
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
      </Box>
    </EditorPopoverAction>
  );
}
