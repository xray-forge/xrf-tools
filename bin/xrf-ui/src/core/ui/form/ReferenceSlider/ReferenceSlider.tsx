import { Slider, SliderProps } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { getSliderReferenceSx } from "@/core/theme/slider-reference";

export interface IReferenceSliderProps extends Omit<SliderProps, "marks" | "color" | "value"> {
  value: number;
  min: number;
  max: number;
  /** The value a reset returns to, marked on the track; none where nothing is measured. */
  reference?: Nullable<number>;
  /** The range xrf recommends, drawn on the rail; none where the setting is not rated. */
  recommended?: Nullable<readonly [number, number]>;
  /** Told on a double click, the reset sliders conventionally take. */
  onReset?: () => void;
}

/**
 * MUI's slider measured by a reference, marked on its track, and a recommended range, its band on the rail; painted as
 * a warning while its value stands outside the range.
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
  const marks: Array<{ value: number }> = useMemo(
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
      sx={[getSliderReferenceSx(recommended, min, max), ...(Array.isArray(sx) ? sx : [sx])]}
      onDoubleClick={onReset}
    />
  );
}
