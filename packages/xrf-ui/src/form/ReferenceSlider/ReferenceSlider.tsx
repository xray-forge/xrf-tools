import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { isSliderWasted } from "#/form/ReferenceSlider/slider-reference";
import { Slider, SliderProps } from "#/material";
import { getSliderReferenceSx } from "#/theme/slider-reference";

export interface IReferenceSliderProps extends Omit<SliderProps, "marks" | "color" | "value"> {
  value: number;
  min: number;
  max: number;
  /** The value a reset returns to, marked on the track; none where nothing is measured. */
  reference?: Nullable<number>;
  /** The range xrf recommends, drawn on the rail; none where the setting is not rated. */
  recommended?: Nullable<readonly [number, number]>;
  /** Past it a higher value only costs more, drawn red on the rail; none where not measured. */
  wastedAbove?: Nullable<number>;
  /** Told on a double click, the reset sliders conventionally take. */
  onReset?: () => void;
}

/**
 * MUI's slider measured by a reference, marked on its track, a recommended range, its band on the rail, and a wasted
 * end, red on the rail; painted as a warning outside the range and as an error past the useful end.
 */
export function ReferenceSlider({
  value,
  min,
  max,
  reference,
  recommended,
  wastedAbove,
  onReset,
  sx,
  ...props
}: IReferenceSliderProps): ReactElement {
  const isOutside: boolean = !!recommended && (value < recommended[0] || value > recommended[1]);
  const isWasted: boolean = isSliderWasted(value, wastedAbove);
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
      color={isWasted ? "error" : isOutside ? "warning" : "primary"}
      sx={[getSliderReferenceSx(recommended, min, max, wastedAbove), ...(Array.isArray(sx) ? sx : [sx])]}
      onDoubleClick={onReset}
    />
  );
}
