import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

/** One value a choice offers, in display order. */
export interface IRenderValueChoiceOption<T extends string> {
  value: T;
  label: string;
}

export interface IRenderValueChoiceProps<T extends string> extends BaseComponentProps {
  label: string;
  options: ReadonlyArray<IRenderValueChoiceOption<T>>;
  value: T;
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
  onChange,
}: IRenderValueChoiceProps<T>): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={className}>
      <span className={"text-xs text-text-secondary"}>{label}</span>

      <ToggleButtonGroup
        className={"mt-1"}
        exclusive
        fullWidth
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
        {options.map((option: IRenderValueChoiceOption<T>) => (
          <ToggleButton key={option.value} value={option.value}>
            {option.label}
          </ToggleButton>
        ))}
      </ToggleButtonGroup>
    </div>
  );
}
