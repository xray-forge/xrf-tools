import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

import { BaseComponentProps } from "#/element-types";
import { TextField } from "#/input";

/** Whether the text is a whole number already, rather than one on its way, such as `-` or `1e`. */
function isNumberText(text: string): boolean {
  return text.trim() !== "" && Number.isFinite(Number(text));
}

export interface INumberFieldProps extends BaseComponentProps {
  label: string;
  value: number;
  isDisabled?: boolean;
  /** Spells a value set from outside; `String` unless the field rounds what it shows. */
  format?: (value: number) => string;
  /** Told each number typed in whole; what is not one yet is held as typed. */
  onCommit: (value: number) => void;
}

/**
 * A number typed in, held as text while it is not a number yet, such as `-`, and marked as an error until it is one.
 */
export function NumberField({
  "data-testid": dataTestId = "number-field",
  id,
  className,
  label,
  value,
  isDisabled = false,
  format = String,
  onCommit,
}: INumberFieldProps): ReactElement {
  const [text, setText] = useState<string>(() => format(value));
  const isNumber: boolean = isNumberText(text);

  const onChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const next: string = event.target.value;

      setText(next);

      if (isNumberText(next)) {
        onCommit(Number(next));
      }
    },
    [onCommit]
  );

  // A value set from elsewhere replaces what was typed; the one just typed keeps its spelling.
  useEffect(() => {
    setText((current: string) => (Number(current) === value ? current : format(value)));
  }, [format, value]);

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      size={"small"}
      type={"number"}
      label={label}
      value={text}
      error={!isNumber}
      disabled={isDisabled}
      slotProps={{ htmlInput: { "aria-label": label, step: "any" } }}
      onChange={onChange}
    />
  );
}
