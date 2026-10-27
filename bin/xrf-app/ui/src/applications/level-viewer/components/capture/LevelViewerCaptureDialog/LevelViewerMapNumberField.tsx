import { BaseComponentProps } from "@xrf/ui/element-types";
import { TextField } from "@xrf/ui/material/text-field";
import { ChangeEvent, ReactElement, useCallback, useEffect, useState } from "react";

/** Metres to a tenth, dropping a trailing `.0`. */
function formatMetres(value: number): string {
  return String(Math.round(value * 10) / 10);
}

interface ILevelViewerMapNumberFieldProps extends BaseComponentProps {
  label: string;
  /** Metres, as the map's parameters hold them. */
  value: number;
  isDisabled?: boolean;
  /** Told each number typed in whole; what is not one yet is held as typed. */
  onCommit: (value: number) => void;
}

/** One of a level map's distances in metres, typed in: held as text while it is not a number yet, such as `-`. */
export function LevelViewerMapNumberField({
  "data-testid": dataTestId = "level-viewer-map-number-field",
  id,
  className,
  label,
  value,
  isDisabled = false,
  onCommit,
}: ILevelViewerMapNumberFieldProps): ReactElement {
  const [text, setText] = useState<string>(() => formatMetres(value));
  const parsed: number = Number(text);
  const isNumber: boolean = text.trim() !== "" && Number.isFinite(parsed);

  const onChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const next: string = event.target.value;
      const number: number = Number(next);

      setText(next);

      if (next.trim() !== "" && Number.isFinite(number)) {
        onCommit(number);
      }
    },
    [onCommit]
  );

  // A value set from elsewhere, such as another area chosen, replaces what was typed.
  useEffect(() => {
    setText((current: string) => (Number(current) === value ? current : formatMetres(value)));
  }, [value]);

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
