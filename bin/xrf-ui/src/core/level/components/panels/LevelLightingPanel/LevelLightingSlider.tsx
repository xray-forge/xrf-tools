import { Slider } from "@mui/material";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ILevelLightingSliderProps extends BaseComponentProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step: number;
  /** How the value reads beside its label, which is the only place a unit is stated. */
  format: (value: number) => string;
  onChange: (value: number) => void;
}

/**
 * One lighting value, with what it currently reads beside its name.
 */
export function LevelLightingSlider({
  "data-testid": dataTestId = "level-lighting-slider",
  id,
  className,
  label,
  value,
  min,
  max,
  step,
  format,
  onChange,
}: ILevelLightingSliderProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <div className={"flex items-baseline justify-between gap-2"}>
        <span className={"text-xs text-text-secondary"}>{label}</span>
        <span className={"font-mono text-xs"}>{format(value)}</span>
      </div>

      <Slider
        size={"small"}
        min={min}
        max={max}
        step={step}
        value={value}
        aria-label={label}
        onChange={(_, next) => onChange(Array.isArray(next) ? next[0] : next)}
      />
    </div>
  );
}
