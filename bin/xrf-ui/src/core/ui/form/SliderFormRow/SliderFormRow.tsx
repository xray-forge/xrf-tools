import { Slider, Typography } from "@mui/material";
import { ReactElement } from "react";

import { FormRow } from "@/core/ui/form/FormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISliderFormRowProps extends BaseComponentProps {
  label: string;
  description?: string;
  value: number;
  min: number;
  max: number;
  step: number;
  /** How the value reads beside the slider, and to a screen reader. */
  format?: (value: number) => string;
  isDisabled?: boolean;
  onChange: (value: number) => void;
}

/** A labelled, controlled number within a range, its value read out beside it. */
export function SliderFormRow({
  "data-testid": dataTestId = "slider-form-row",
  id,
  className,
  label,
  description,
  value,
  min,
  max,
  step,
  format = String,
  isDisabled = false,
  onChange,
}: ISliderFormRowProps): ReactElement {
  return (
    <FormRow label={label} description={description} controlId={id}>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <div className={"flex items-center gap-4"}>
          <Slider
            data-testid={dataTestId}
            id={controlId}
            className={className}
            size={"small"}
            value={value}
            min={min}
            max={max}
            step={step}
            disabled={isDisabled}
            getAriaValueText={format}
            slotProps={{ input: { "aria-labelledby": labelId, "aria-describedby": describedBy } }}
            onChange={(_, next: number | Array<number>) => onChange(Array.isArray(next) ? next[0] : next)}
          />
          <Typography className={"w-16 shrink-0 text-right tabular-nums"} variant={"body2"}>
            {format(value)}
          </Typography>
        </div>
      )}
    </FormRow>
  );
}
