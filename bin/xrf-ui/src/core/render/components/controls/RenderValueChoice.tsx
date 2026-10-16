import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IRenderValueChoiceProps<T extends string> extends BaseComponentProps {
  label: string;
  options: ReadonlyArray<IRenderChoiceOption<T>>;
  value: T;
  /** Whether the values stack one under another, for names too long to share a row. */
  isStacked?: boolean;
  onChange: (value: T) => void;
}

/**
 * One of a few values a preview is driven by, picked from a row that keeps one selected.
 */
export function RenderValueChoice<T extends string>({
  "data-testid": dataTestId = "render-value-choice",
  id,
  className,
  label,
  options,
  value,
  isStacked = false,
  onChange,
}: IRenderValueChoiceProps<T>): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <span className={"text-xs text-text-secondary"}>{label}</span>

      <ToggleButtonGroup
        className={"mt-1"}
        exclusive
        fullWidth
        orientation={isStacked ? "vertical" : "horizontal"}
        color={"primary"}
        size={"small"}
        value={value}
        aria-label={label}
        onChange={(_, next: Nullable<T>) => {
          if (next !== null) {
            onChange(next);
          }
        }}
      >
        {options.map((option: IRenderChoiceOption<T>) => (
          <ToggleButton key={option.value} value={option.value} disabled={option.isDisabled}>
            {option.label}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    </div>
  );
}
