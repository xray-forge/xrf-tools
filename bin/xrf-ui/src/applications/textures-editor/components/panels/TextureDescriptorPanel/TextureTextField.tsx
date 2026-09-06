import { TextField } from "@mui/material";
import { ChangeEvent, ReactElement, useCallback } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

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
  const handleChange = useCallback((event: ChangeEvent<HTMLInputElement>) => onChange(event.target.value), [onChange]);

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
      onChange={handleChange}
    />
  );
}
