import { MenuItem, TextField } from "@mui/material";
import { ChangeEvent, ReactElement, useCallback } from "react";

import { toVocabularyOptions } from "@/applications/textures-editor/lib/texture-vocabulary";
import { TextureVocabularyEntry } from "@/core/ipc/types/xrf-app";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ITextureSelectFieldProps extends BaseComponentProps {
  label: string;
  /** The named values the SDK offers for this field. */
  entries: ReadonlyArray<TextureVocabularyEntry>;
  value: number;
  helperText?: string;
  onChange: (value: number) => void;
}

/**
 * One enumerated descriptor field, offering the values the SDK names plus whatever this file happens to hold.
 */
export function TextureSelectField({
  "data-testid": dataTestId = "texture-select-field",
  id,
  className,
  label,
  entries,
  value,
  helperText,
  onChange,
}: ITextureSelectFieldProps): ReactElement {
  const handleChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => onChange(Number(event.target.value)),
    [onChange]
  );

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      select
      fullWidth
      size={"small"}
      margin={"dense"}
      label={label}
      value={String(value)}
      helperText={helperText}
      onChange={handleChange}
    >
      {toVocabularyOptions(entries, value).map((entry: TextureVocabularyEntry) => (
        <MenuItem key={entry.value} value={String(entry.value)}>
          {entry.label}
        </MenuItem>
      ))}
    </TextField>
  );
}
