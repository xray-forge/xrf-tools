import { Slider } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { IThrottledDraft, useThrottledDraft } from "@/lib/react/use-throttled-draft";

export interface IRenderValueSliderProps extends BaseComponentProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  /** How the value reads beside its label, which is the only place a unit is stated. */
  format: (value: number) => string;
  /** Told the value while it is dragged at most once an interval, and at once when it is let go. */
  onChange: (value: number) => void;
}

/**
 * One numeric value a preview is driven by, with what it currently reads beside its name.
 */
export function RenderValueSlider({
  "data-testid": dataTestId = "render-value-slider",
  id,
  className,
  label,
  value,
  min,
  max,
  step,
  format,
  onChange,
}: IRenderValueSliderProps): ReactElement {
  const draft: IThrottledDraft = useThrottledDraft(value, step, onChange);

  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <div className={"flex items-baseline justify-between gap-2"}>
        <span className={"text-xs text-text-secondary"}>{label}</span>
        <span className={"font-mono text-xs"}>{format(draft.value)}</span>
      </div>

      <Slider
        size={"small"}
        min={min}
        max={max}
        step={step}
        value={draft.value}
        aria-label={label}
        onChange={draft.onChange}
        onChangeCommitted={draft.onChangeCommitted}
      />
    </div>
  );
}
