import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useState } from "react";

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

/** A number as the field spells it: empty for none. */
function toText(value: Nullable<number>): string {
  return value === null ? "" : String(value);
}

/**
 * One typed number in a row of a toolbar popover or panel, laid out as `EditorPopoverTextRow`. A field mid-edit that
 * reads empty or as a lone minus sign tells the owner nothing.
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
  // The field shows what the hand typed, not the owner's number: a controlled input whose value did not change would be
  // set back to it after each keystroke, so an emptied field or a lone minus sign could never be typed.
  const [text, setText] = useState<string>(() => toText(value));

  // The owner's number over what was typed, unless what was typed spells it.
  const reconcile = useCallback(
    () => setText((current: string) => (current.trim() && Number(current) === value ? current : toText(value))),
    [value]
  );

  // A value set from elsewhere replaces what was typed; the one just typed keeps its spelling.
  useEffect(reconcile, [reconcile]);

  const onTextChange = useCallback(
    (typed: string) => {
      const parsed: number = Number(typed);

      setText(typed);

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
      value={text}
      isReadOnly={isReadOnly}
      isDisabled={isDisabled}
      onChange={onChange ? onTextChange : undefined}
      // An edit left empty, half typed, or at a number the owner refused shows the owner's value again.
      onBlur={reconcile}
    />
  );
}
