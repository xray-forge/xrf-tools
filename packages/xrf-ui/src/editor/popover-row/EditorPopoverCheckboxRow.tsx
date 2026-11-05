import { ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { Checkbox } from "#/material/checkbox";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

export interface IEditorPopoverCheckboxRowProps extends BaseComponentProps {
  label: string;
  /** What it does, read on hovering the label. */
  description?: string;
  isChecked: boolean;
  isDisabled?: boolean;
  onChange: (isChecked: boolean) => void;
}

/** A setting of a toolbar popover that is on or off, its box in the column the other rows' controls start at. */
export function EditorPopoverCheckboxRow({
  "data-testid": dataTestId = "editor-popover-checkbox-row",
  id,
  className,
  label,
  description,
  isChecked,
  isDisabled = false,
  onChange,
}: IEditorPopoverCheckboxRowProps): ReactElement {
  return (
    <EditorPopoverRow id={id} className={className} label={label} description={description} isDisabled={isDisabled}>
      {({ id: controlId, ...names }: IEditorPopoverRowControlProps) => (
        <Checkbox
          data-testid={dataTestId}
          id={controlId}
          // The box's own inset, taken back so it starts where a slider's rail does.
          className={"-ml-1 p-0.5"}
          size={"small"}
          checked={isChecked}
          disabled={isDisabled}
          slotProps={{ input: names }}
          onChange={(_, checked: boolean) => onChange(checked)}
        />
      )}
    </EditorPopoverRow>
  );
}
