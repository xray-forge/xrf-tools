import { ChangeEvent, ReactElement } from "react";

import { BaseComponentProps } from "#/element-types";
import { TextField } from "#/material/text-field";
import { POPOVER_INPUT_SX } from "#/theme/sx";

import { EditorPopoverRow, IEditorPopoverRowControlProps } from "./EditorPopoverRow";

export interface IEditorPopoverTextRowProps extends BaseComponentProps {
  label: string;
  /** What it holds, read on hovering the label and by a screen reader. */
  description?: string;
  /**
   * A number field's value is given as a number, so React compares it numerically and leaves a half-typed `0.0` alone
   * rather than writing `0` back over it; an empty string shows it empty.
   */
  value: string | number;
  /** A number field takes digits and steps by the arrows; the value still comes back as typed. */
  type?: "text" | "number";
  /** Shown and selectable but not edited, such as a value another file is the authority for. */
  isReadOnly?: boolean;
  isDisabled?: boolean;
  /** Absent for a value that is only shown, which reads it read-only. */
  onChange?: (value: string) => void;
}

/**
 * One typed value in a single row of a toolbar popover or panel: the label in the column every row shares, the field
 * beside it as low as the row's other controls.
 */
export function EditorPopoverTextRow({
  "data-testid": dataTestId = "editor-popover-text-row",
  id,
  className,
  label,
  description,
  value,
  type = "text",
  isReadOnly = false,
  isDisabled = false,
  onChange,
}: IEditorPopoverTextRowProps): ReactElement {
  return (
    <EditorPopoverRow id={id} className={className} label={label} description={description} isDisabled={isDisabled}>
      {({
        id: controlId,
        "aria-labelledby": labelId,
        "aria-describedby": descriptionId,
      }: IEditorPopoverRowControlProps) => (
        <TextField
          data-testid={dataTestId}
          id={controlId}
          fullWidth
          size={"small"}
          type={type}
          value={value}
          disabled={isDisabled}
          slotProps={{
            input: { readOnly: isReadOnly || !onChange, sx: POPOVER_INPUT_SX },
            htmlInput: {
              "aria-labelledby": labelId,
              "aria-describedby": descriptionId,
              spellCheck: false,
              step: type === "number" ? "any" : undefined,
            },
          }}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onChange?.(event.target.value)}
        />
      )}
    </EditorPopoverRow>
  );
}
