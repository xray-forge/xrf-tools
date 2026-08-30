import { Slider } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * Kills the easing MUI gives a slider's thumb and track.
 */
const PLAYHEAD_TRANSITION = { "& .MuiSlider-thumb, & .MuiSlider-track": { transition: "none" } } as const;

export interface IMotionFrameSliderProps extends BaseComponentProps {
  /** Frames the posed motion holds. Zero disables the control, because there is no frame to be on. */
  frameCount: number;
  /** Frame on screen, counted from zero. */
  frame: number;
  /** What the control is called, since a surface may play a motion or a track of them. */
  ariaLabel: string;
  onSeek: (frame: number) => void;
}

/**
 * The frame a motion is showing, and the handle that moves it.
 */
export function MotionFrameSlider({
  "data-testid": dataTestId = "motion-frame-slider",
  id,
  className,
  sx,
  ariaLabel,
  frameCount,
  frame,
  onSeek,
}: IMotionFrameSliderProps): ReactElement {
  const onChange = useCallback((_: Event, value: number | Array<number>) => onSeek(value as number), [onSeek]);

  return (
    <Slider
      data-testid={dataTestId}
      id={id}
      className={className}
      aria-label={ariaLabel}
      size={"small"}
      min={0}
      max={Math.max(0, frameCount - 1)}
      value={frame}
      disabled={!frameCount}
      sx={[PLAYHEAD_TRANSITION, ...(Array.isArray(sx) ? sx : [sx])]}
      onChange={onChange}
    />
  );
}
