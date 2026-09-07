import { ReactElement, useId } from "react";

import { FilePickerInput } from "@/core/ui/form/file-picker/FilePickerInput";
import { useCommitOnSubmit } from "@/core/ui/form/form-commit";
import { FormRow } from "@/core/ui/form/FormRow";
import { IPathField } from "@/core/ui/form/use-path-field";

interface IPathFormRowProps {
  label: string;
  description?: string;
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
  isRequired = true,
  isDisabled,
  placeholder,
  field,
}: IPathFormRowProps): ReactElement {
  const controlId: string = useId();

  useCommitOnSubmit(field.commit);

  return (
    <FormRow label={label} description={description} isRequired={isRequired} error={field.error} controlId={controlId}>
      <FilePickerInput
        id={controlId}
        placeholder={placeholder}
        value={field.value}
        isDisabled={isDisabled}
        isInvalid={Boolean(field.error)}
        recents={field.recents}
        onSelect={field.select}
        onChange={field.setValue}
        onClear={field.clear}
      />
    </FormRow>
  );
}
