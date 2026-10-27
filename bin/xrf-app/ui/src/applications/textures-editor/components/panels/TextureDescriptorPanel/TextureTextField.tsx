import { BaseComponentProps } from "@xrf/ui/element-types";
import { TextField } from "@xrf/ui/material";
import { ChangeEvent, ReactElement, useCallback } from "react";

interface ITextureTextFieldProps extends BaseComponentProps {
  label: string;
  value: string;
  helperText?: string;
  onChange: (value: string) => void;
}

/**
 * One free-text descriptor field, such as an engine reference the descriptor names.
 */
export function TextureTextField({
  "data-testid": dataTestId = "texture-text-field",
  id,
  className,
  label,
  value,
  helperText,
  onChange,
}: ITextureTextFieldProps): ReactElement {
  const onFieldChange = useCallback((event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value), [onChange]);

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      fullWidth
      size={"small"}
      margin={"dense"}
      label={label}
      value={value}
      helperText={helperText}
      onChange={onFieldChange}
    />
  );
}
