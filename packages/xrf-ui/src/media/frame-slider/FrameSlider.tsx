import { ReactElement, useCallback } from "react";

import { BaseComponentProps } from "#/element-types";
import { Slider, sliderClasses } from "#/form/control";

/**
 * Kills the easing MUI gives a slider's thumb and track.
 */
const PLAYHEAD_TRANSITION = {
  [`& .${sliderClasses.thumb}, & .${sliderClasses.track}`]: { transition: "none" },
} as const;

export interface IFrameSliderProps extends BaseComponentProps {
  /** Frames the timeline holds. Zero disables the control, because there is no frame to be on. */
  frameCount: number;
  /** Frame on screen, counted from zero. */
  frame: number;
  /** What the control is called, since one surface may play a single motion and another a track of them. */
  ariaLabel: string;
  onSeek: (frame: number) => void;
}

/**
 * The frame a timeline is showing, and the playhead that moves it.
 */
export function FrameSlider({
  "data-testid": dataTestId = "frame-slider",
  id,
  className,
  ariaLabel,
  frameCount,
  frame,
  onSeek,
}: IFrameSliderProps): ReactElement {
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
      sx={PLAYHEAD_TRANSITION}
      onChange={onChange}
    />
  );
}
