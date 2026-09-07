import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as HistoryIcon } from "@mui/icons-material/History";
import { Box, IconButton, TextField, Tooltip } from "@mui/material";
import { ChangeEvent, ReactElement, useId, useState } from "react";

import { MONOSPACE } from "@/core/theme/tokens";
import { FilePickerRecentsMenu } from "@/core/ui/form/file-picker/FilePickerRecentsMenu";
import { FormRow } from "@/core/ui/form/FormRow";
import { IPathFieldRecents } from "@/core/ui/form/path-recents";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface IFilePickerInputProps extends BaseComponentProps {
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
  /** Paths this field was given before. Absent or empty, and the control offers no history at all. */
  recents?: IPathFieldRecents;
}

/**
 * The control half of a path row.
 *
 * The value is monospaced because these are filesystem paths, compared by eye.
 */
export function FilePickerInput({
  "data-testid": dataTestId,
  id,
  className,
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
  recents,
}: IFilePickerInputProps): ReactElement {
  const generatedId: string = useId();
  const controlId: string = id ?? generatedId;

  // The field itself anchors the history, not the button that opens it: a list of paths belongs under the control
  // whose width it is measured against, the way any other combo box drops.
  const [field, setField] = useState<Nullable<HTMLElement>>(null);
  const [isRecentsOpen, setRecentsOpen] = useState<boolean>(false);

  // An empty history offers nothing, so it says nothing: a permanent button that opens a permanently empty menu reads
  // as a broken feature rather than as an unused one.
  const hasRecents: boolean = Boolean(recents?.records.length);

  const control: ReactElement = (
    <TextField
      ref={setField}
      data-testid={dataTestId}
      id={controlId}
      className={className}
      fullWidth
      size={"small"}
      placeholder={placeholder}
      disabled={isDisabled}
      error={isInvalid}
      value={value ?? ""}
      sx={{ "& .MuiInputBase-input": MONOSPACE }}
      slotProps={{
        htmlInput: {
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

              {hasRecents ? (
                <Tooltip describeChild title={"Recent paths"}>
                  <span>
                    <IconButton
                      aria-label={"Recent paths"}
                      disabled={isDisabled}
                      onClick={() => setRecentsOpen(true)}
                    >
                      <HistoryIcon fontSize={"small"} />
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

  // Beside the field rather than inside its adornment: the menu is a portal either way, and nesting a popover in the
  // input put its own focus trap inside the control.
  const controlWithHistory: ReactElement = (
    <>
      {control}

      {recents && hasRecents ? (
        <FilePickerRecentsMenu
          isOpen={isRecentsOpen}
          anchor={field}
          recents={recents}
          currentPath={value}
          onClose={() => setRecentsOpen(false)}
        />
      ) : null}
    </>
  );

  return label ? (
    <FormRow label={label} description={description} isRequired={isRequired} error={error} controlId={controlId}>
      {controlWithHistory}
    </FormRow>
  ) : (
    controlWithHistory
  );
}
