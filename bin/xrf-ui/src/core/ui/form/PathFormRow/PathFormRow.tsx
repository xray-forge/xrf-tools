import { ReactElement } from "react";

import { FilePickerInput } from "@/core/ui/form/file-picker/FilePickerInput";
import { useCommitOnSubmit } from "@/core/ui/form/form-commit";
import { FormRow } from "@/core/ui/form/FormRow";
import { IPathField } from "@/core/ui/form/use-path-field";
import { Nullable } from "@/lib/types/general";

interface IPathFormRowProps {
  label: string;
  description?: string;
  /** Describes the current path when there is no validation error. */
  fact?: Nullable<string>;
  isRequired?: boolean;
  isDisabled?: boolean;
  placeholder?: string;
  field: IPathField;
}

/**
 * A labelled, validated, remembered path - the whole row in one element.
 *
 * Also where the field joins the enclosing form, because the row is inside it and the hook that owns the field is
 * called above it. A row outside a form joins nothing, which is why a screen with its own run action commits itself.
 */
export function PathFormRow({
  label,
  description,
  fact,
  isRequired = true,
  isDisabled,
  placeholder,
  field,
}: IPathFormRowProps): ReactElement {
  useCommitOnSubmit(field.commit);

  return (
    <FormRow label={label} description={description} isRequired={isRequired} error={field.error} fact={fact}>
      {({ "aria-describedby": describedBy, "aria-invalid": isInvalid, id }) => (
        <FilePickerInput
          id={id}
          aria-describedby={describedBy}
          placeholder={placeholder}
          value={field.value}
          isDisabled={isDisabled}
          isInvalid={isInvalid}
          recents={field.recents}
          onSelect={field.select}
          onChange={field.setValue}
          onClear={field.clear}
        />
      )}
    </FormRow>
  );
}
