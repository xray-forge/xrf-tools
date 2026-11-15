import { Nullable } from "@xrf/types";
import { ReactElement, useCallback } from "react";

import { BaseComponentProps } from "#/element-types";

import { EditorPopoverTextRow } from "./EditorPopoverTextRow";

export interface IEditorPopoverNumberRowProps extends BaseComponentProps {
  label: string;
  /** What it holds, read on hovering the label and by a screen reader. */
  description?: string;
  /** Null for a value there is no number for, such as a non-finite one a file held; it shows empty. */
  value: Nullable<number>;
  /** Shown and selectable but not edited, such as a value another file is the authority for. */
  isReadOnly?: boolean;
  isDisabled?: boolean;
  /** Told each finite number typed; absent for a value that is only shown, which reads it read-only. */
  onChange?: (value: number) => void;
}

/**
 * One typed number in a single row of a toolbar popover or panel, as `EditorPopoverTextRow` lays it, told only numbers:
 * a field mid-edit that reads empty or as a lone minus sign is left to the hand.
 */
export function EditorPopoverNumberRow({
  "data-testid": dataTestId = "editor-popover-number-row",
  id,
  className,
  label,
  description,
  value,
  isReadOnly = false,
  isDisabled = false,
  onChange,
}: IEditorPopoverNumberRowProps): ReactElement {
  const onTextChange = useCallback(
    (typed: string) => {
      const parsed: number = Number(typed);

      // An empty field parses to zero and a lone minus sign to NaN: neither is a number the hand has finished typing,
      // and either would be written into the owner's value.
      if (onChange && typed.trim() && Number.isFinite(parsed)) {
        onChange(parsed);
      }
    },
    [onChange]
  );

  return (
    <EditorPopoverTextRow
      data-testid={dataTestId}
      id={id}
      className={className}
      label={label}
      description={description}
      type={"number"}
      value={value ?? ""}
      isReadOnly={isReadOnly}
      isDisabled={isDisabled}
      onChange={onChange ? onTextChange : undefined}
    />
  );
}
