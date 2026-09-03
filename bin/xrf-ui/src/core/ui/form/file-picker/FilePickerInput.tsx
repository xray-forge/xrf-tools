import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Box, IconButton, TextField, Tooltip } from "@mui/material";
import { ChangeEvent, ReactElement, useId } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
import { FormRow } from "@/core/ui/form/FormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IFilePickerInputProps extends BaseComponentProps {
  /** When given, the control labels itself by composing a `FormRow`. */
  label?: string;
  description?: string;
  isRequired?: boolean;
  error?: Nullable<string>;
  placeholder?: string;
  value?: Nullable<string>;
  /** Ties this control to a label a caller already rendered. */
  isDisabled?: boolean;
  isInvalid?: boolean;
  onSelect: () => void;
  /** Enables typing and pasting a path. Without it the field only reports what the dialog returned. */
  onChange?: (value: string) => void;
  onClear?: () => void;
}

/**
 * The control half of a path row.
 *
 * The value is monospaced because these are filesystem paths, compared by eye.
 */
export function FilePickerInput({
  id,
  label,
  description,
  isRequired,
  error,
  placeholder = "Not selected",
  value,
  isDisabled,
  isInvalid,
  onSelect,
  onChange,
  onClear,
}: IFilePickerInputProps): ReactElement {
  const generatedId: string = useId();
  const controlId: string = id ?? generatedId;

  const control: ReactElement = (
    <TextField
      fullWidth
      size={"small"}
      placeholder={placeholder}
      disabled={isDisabled}
      error={isInvalid}
      value={value ?? ""}
      sx={{ "& .MuiInputBase-input": MONOSPACE }}
      slotProps={{
        htmlInput: {
          id: controlId,
          spellCheck: false,
          // Paths are compared and edited from the end far more often than from the start.
          autoComplete: "off",
        },
        input: {
          readOnly: !onChange,
          endAdornment: (
            <Box sx={{ display: "flex", flexShrink: 0 }}>
              {value && onClear ? (
                <Tooltip describeChild title={"Clear"}>
                  <span>
                    <IconButton aria-label={"Clear"} disabled={isDisabled} onClick={onClear}>
                      <ClearIcon fontSize={"small"} />
                    </IconButton>
                  </span>
                </Tooltip>
              ) : null}

              <Tooltip describeChild title={"Browse"}>
                <span>
                  <IconButton aria-label={"Browse"} disabled={isDisabled} onClick={onSelect}>
                    <FolderOpenIcon fontSize={"small"} />
                  </IconButton>
                </span>
              </Tooltip>
            </Box>
          ),
        },
      }}
      onChange={(event: ChangeEvent<HTMLInputElement>) => onChange?.(event.target.value)}
    />
  );

  return label ? (
    <FormRow label={label} description={description} isRequired={isRequired} error={error} controlId={controlId}>
      {control}
    </FormRow>
  ) : (
    control
  );
}
