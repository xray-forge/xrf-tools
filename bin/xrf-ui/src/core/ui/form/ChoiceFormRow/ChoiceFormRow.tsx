import { ToggleButton, ToggleButtonGroup } from "@mui/material";
import { ReactElement } from "react";

import { FormRow } from "@/core/ui/form/FormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** One value offered by a choice row, in display order. */
export interface IChoiceFormRowOption<T extends string> {
  value: T;
  label: string;
  /** Accessible name when the visible label needs more context. */
  "aria-label"?: string;
}

export interface IChoiceFormRowProps<T extends string> extends BaseComponentProps {
  label: string;
  description?: string;
  options: ReadonlyArray<IChoiceFormRowOption<T>>;
  value: T;
  isRequired?: boolean;
  isDisabled?: boolean;
  onChange: (value: T) => void;
}

/** A labelled, controlled choice that keeps one option selected. */
export function ChoiceFormRow<T extends string>({
  "data-testid": dataTestId = "choice-form-row",
  id,
  className,
  label,
  description,
  options,
  value,
  isRequired = true,
  isDisabled = false,
  onChange,
}: IChoiceFormRowProps<T>): ReactElement {
  return (
    <FormRow label={label} description={description} controlId={id} isRequired={isRequired} isGroup>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <ToggleButtonGroup
          data-testid={dataTestId}
          id={controlId}
          className={className}
          aria-labelledby={labelId}
          aria-describedby={describedBy}
          exclusive
          color={"primary"}
          size={"small"}
          value={value}
          disabled={isDisabled}
          onChange={(_, next: Nullable<T>) => {
            if (next !== null) {
              onChange(next);
            }
          }}
        >
          {options.map((option) => (
            <ToggleButton key={option.value} aria-label={option["aria-label"]} value={option.value}>
              {option.label}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      )}
    </FormRow>
  );
}
