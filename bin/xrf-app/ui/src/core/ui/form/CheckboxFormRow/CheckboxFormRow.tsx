import { Checkbox } from "@mui/material";
import { ReactElement } from "react";

import { FormRow } from "@/core/ui/form/FormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ICheckboxFormRowProps extends BaseComponentProps {
  label: string;
  description?: string;
  isChecked: boolean;
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
          disabled={isDisabled}
          onChange={(_, checked) => onChange(checked)}
        />
      )}
    </FormRow>
  );
}
