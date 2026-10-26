import { Switch } from "@mui/material";
import { ReactElement } from "react";

import { FormRow } from "@/core/ui/form/FormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface ISwitchFormRowProps extends BaseComponentProps {
  label: string;
  description?: string;
  isChecked: boolean;
  isDisabled?: boolean;
  /** Whether a run needs the choice: a row that does not is marked optional. */
  isRequired?: boolean;
  onChange: (isChecked: boolean) => void;
}

/** A controlled switch with its label and explanation linked to the input. */
export function SwitchFormRow({
  "data-testid": dataTestId = "switch-form-row",
  id,
  className,
  label,
  description,
  isChecked,
  isDisabled = false,
  isRequired = true,
  onChange,
}: ISwitchFormRowProps): ReactElement {
  return (
    <FormRow label={label} description={description} controlId={id} isRequired={isRequired} isInline>
      {({ id: controlId, "aria-labelledby": labelId, "aria-describedby": describedBy }) => (
        <Switch
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
