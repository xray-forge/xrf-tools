import { Slider, sliderClasses, SliderProps, Theme } from "@mui/material";
import { SystemStyleObject } from "@mui/system";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

export interface IReferenceSliderProps extends Omit<SliderProps, "marks" | "color" | "value"> {
  value: number;
  min: number;
  max: number;
  /** The value a reset returns to, marked on the track; none where nothing is measured. */
  reference?: Nullable<number>;
  /** The range xrf recommends, drawn on the rail; none where the setting is not rated. */
  recommended?: Nullable<readonly [number, number]>;
  /** Told the reference on a double click, the reset sliders conventionally take. */
  onReset?: () => void;
}

/**
 * MUI's slider with what it is measured by: the reference value as a notch on the track, and the recommended range as
 * a green band on a rail taller than the track, so it shows behind the filled part too, and the slider painted in the
 * warning's yellow-gold while its value stands outside it.
 */
export function ReferenceSlider({
  value,
  min,
  max,
  reference,
  recommended,
  onReset,
  sx,
  ...props
}: IReferenceSliderProps): ReactElement {
  const isOutside: boolean = !!recommended && (value < recommended[0] || value > recommended[1]);
  const marks = useMemo(
    () =>
      reference !== null && reference !== undefined && reference >= min && reference <= max
        ? [{ value: reference }]
        : [],
    [reference, min, max]
  );

  return (
    <Slider
      {...props}
      value={value}
      min={min}
      max={max}
      marks={marks}
      color={isOutside ? "warning" : "primary"}
      sx={[(theme: Theme) => getReferenceSx(theme, recommended, [min, max]), ...(Array.isArray(sx) ? sx : [sx])]}
      onDoubleClick={onReset}
    />
  );
}

/**
 * @returns The rail as the recommended range paints it, and the reference's notch standing out of the track.
 */
function getReferenceSx(
  theme: Theme,
  recommended: Nullable<readonly [number, number]> | undefined,
  [min, max]: readonly [number, number]
): SystemStyleObject<Theme> {
  const palette = (theme.vars ?? theme).palette;
  const notch: SystemStyleObject<Theme> = {
    [`& .${sliderClasses.mark}`]: {
      backgroundColor: palette.text.secondary,
      height: 12,
      width: 2,
      borderRadius: "1px",
      opacity: 1,
    },
    [`& .${sliderClasses.markActive}`]: { backgroundColor: palette.text.secondary, opacity: 1 },
  };

  if (!recommended || max <= min) {
    return notch;
  }

  // Outside the range the rail is a rail as any other slider's; the warning is the slider's own colour, which only a
  // value standing there takes, as an amber-branded window cannot also warn in amber on the rail.
  const outside: string = `color-mix(in srgb, ${palette.text.primary} 20%, transparent)`;
  const inside: string = `color-mix(in srgb, ${palette.success.main} 55%, transparent)`;
  const low: string = toTrackShare(recommended[0], min, max);
  const high: string = toTrackShare(recommended[1], min, max);

  return {
    ...notch,
    [`& .${sliderClasses.rail}`]: {
      opacity: 1,
      height: 6,
      borderRadius: "3px",
      background: `linear-gradient(to right, ${outside} 0 ${low}, ${inside} ${low} ${high}, ${outside} ${high} 100%)`,
    },
  };
}

/** Where a value stands along a track from `min` to `max`, as a CSS percentage held to the track. */
function toTrackShare(at: number, min: number, max: number): string {
  return `${(Math.min(Math.max((at - min) / (max - min), 0), 1) * 100).toFixed(2)}%`;
}
