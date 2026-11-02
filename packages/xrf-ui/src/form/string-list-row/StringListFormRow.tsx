import { withoutAt, withValueAt } from "@xrf/collections";
import { ChangeEvent, ReactElement } from "react";

import { EditableList, EditableListItem } from "#/form/editable-list";
import { FormRow } from "#/form/row";
import { TextField } from "#/material/text-field";

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
              slotProps={{ htmlInput: { "aria-label": `${addLabel} ${index + 1}` } }}
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
