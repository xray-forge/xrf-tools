import { TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { EditableList, EditableListItem } from "@/core/ui/form/EditableList";
import { FormRow } from "@/core/ui/form/FormRow";
import { withoutAt, withValueAt } from "@/lib/types/array";

interface IStringListFormRowProps {
  label: string;
  description?: string;
  values: Array<string>;
  addLabel: string;
  emptyLabel: string;
  placeholder?: string;
  isDisabled?: boolean;
  onChange: (values: Array<string>) => void;
}

/**
 * A labelled string list that preserves entry order without validating values.
 */
export function StringListFormRow({
  label,
  description,
  values,
  addLabel,
  emptyLabel,
  placeholder,
  isDisabled,
  onChange,
}: IStringListFormRowProps): ReactElement {
  return (
    <FormRow label={label} description={description}>
      <EditableList
        addLabel={addLabel}
        emptyLabel={emptyLabel}
        isDisabled={isDisabled}
        onAdd={() => onChange([...values, ""])}
      >
        {values.map((value, index) => (
          <EditableListItem
            key={index}
            removeLabel={`Remove ${value || `entry ${index + 1}`}`}
            isDisabled={isDisabled}
            onRemove={() => onChange(withoutAt(values, index))}
          >
            <TextField
              size={"small"}
              fullWidth
              value={value}
              placeholder={placeholder}
              disabled={isDisabled}
              onChange={(event: ChangeEvent<HTMLInputElement>) =>
                onChange(withValueAt(values, index, event.target.value))
              }
            />
          </EditableListItem>
        ))}
      </EditableList>
    </FormRow>
  );
}
