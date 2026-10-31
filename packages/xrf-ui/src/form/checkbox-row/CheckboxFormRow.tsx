import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { FormRow } from "#/form/row";
import { Checkbox } from "#/material/checkbox";

export interface ICheckboxFormRowProps extends BaseComponentProps {
  label: string;
  description?: string;
  isChecked: boolean;
  /** Shows neither checked nor clear, for a switch over several of which only some are on. */
  isIndeterminate?: boolean;
  isDisabled?: boolean;
  onChange: (isChecked: boolean) => void;
}

/** A controlled checkbox with its label and explanation linked to the input. */
export function CheckboxFormRow({
  "data-testid": dataTestId = "checkbox-form-row",
  id,
  className,
  label,
  description,
  isChecked,
  isIndeterminate = false,
  isDisabled = false,
  onChange,
}: ICheckboxFormRowProps): ReactElement {
  return (
    <FormRow label={label} description={description} controlId={id} isInline>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <Checkbox
          data-testid={dataTestId}
          id={controlId}
          className={className}
          slotProps={{ input: { "aria-labelledby": labelId, "aria-describedby": describedBy } }}
          size={"small"}
          checked={isChecked}
          indeterminate={isIndeterminate}
          disabled={isDisabled}
          onChange={(_, checked) => onChange(checked)}
        />
      )}
    </FormRow>
  );
}
