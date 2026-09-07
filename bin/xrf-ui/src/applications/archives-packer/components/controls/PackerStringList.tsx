import { TextField } from "@mui/material";
import { ChangeEvent, ReactElement } from "react";

import { withoutAt, withValueAt } from "@/applications/archives-packer/lib/pack-config";

import { PackerEditableList } from "./PackerEditableList";
import { PackerEditableRow } from "./PackerEditableRow";

interface IPackerStringListProps {
  values: Array<string>;
  isDisabled?: boolean;
  addLabel: string;
  emptyLabel: string;
  placeholder?: string;
  onChange: (values: Array<string>) => void;
}

/**
 * Editable list of plain strings, for the sections that are just names or patterns.
 */
export function PackerStringList({
  values,
  isDisabled,
  addLabel,
  emptyLabel,
  placeholder,
  onChange,
}: IPackerStringListProps): ReactElement {
  return (
    <PackerEditableList
      addLabel={addLabel}
      emptyLabel={emptyLabel}
      isDisabled={isDisabled}
      onAdd={() => onChange([...values, ""])}
    >
      {values.map((value, index) => (
        <PackerEditableRow
          key={index}
          removeLabel={`Remove ${value || `entry ${index + 1}`}`}
          isDisabled={isDisabled}
          onRemove={() => onChange(withoutAt(values, index))}
        >
          <TextField
            size={"small"}
            fullWidth
            disabled={isDisabled}
            value={value}
            placeholder={placeholder}
            slotProps={{ htmlInput: { "aria-label": `${addLabel} ${index + 1}` } }}
            onChange={(event: ChangeEvent<HTMLInputElement>) =>
              onChange(withValueAt(values, index, event.target.value))
            }
          />
        </PackerEditableRow>
      ))}
    </PackerEditableList>
  );
}
