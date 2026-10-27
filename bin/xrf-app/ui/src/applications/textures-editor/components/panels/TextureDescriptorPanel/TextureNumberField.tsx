import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { TextField } from "@xrf/ui/material";
import { ChangeEvent, ReactElement, useCallback } from "react";

interface ITextureNumberFieldProps extends BaseComponentProps {
  label: string;
  /**
   * Null where the descriptor holds a value JSON cannot carry, which for an `f32` means non-finite.
   */
  value: Nullable<number>;
  helperText?: string;
  isReadOnly?: boolean;
  onChange?: (value: number) => void;
}

/**
 * One numeric descriptor field.
 */
export function TextureNumberField({
  "data-testid": dataTestId = "texture-number-field",
  id,
  className,
  label,
  value,
  helperText,
  isReadOnly = false,
  onChange,
}: ITextureNumberFieldProps): ReactElement {
  const handleChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      const parsed: number = Number(event.target.value);

      // A field mid-edit can read as empty or as a lone minus sign, which parse to NaN and would otherwise be written
      // into the draft and then into the file.
      if (onChange && Number.isFinite(parsed)) {
        onChange(parsed);
      }
    },
    [onChange]
  );

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      fullWidth
      size={"small"}
      margin={"dense"}
      type={"number"}
      label={label}
      value={value ?? ""}
      helperText={helperText}
      slotProps={{ input: { readOnly: isReadOnly || !onChange } }}
      onChange={handleChange}
    />
  );
}
