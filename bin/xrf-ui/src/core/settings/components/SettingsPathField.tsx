import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Box, IconButton, TextField, Tooltip } from "@mui/material";
import { ReactElement } from "react";

import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { withStoppedPropagation } from "@/lib/dom/event";
import { Nullable } from "@/lib/types/general";

export interface ISettingsPathFieldProps {
  label: string;
  description: string;
  value: Nullable<string>;
  /**
   * The path this field would derive while nothing is set, shown greyed in place of the value.
   */
  placeholder?: Nullable<string>;
  /** What the current value turned out to be, stated beside the label. */
  fact?: Nullable<string>;
  onSelect: () => void;
  onClear: () => void;
}

/**
 * One directory setting: what it is, what it is set to, and the two things you can do to it.
 */
export function SettingsPathField({
  label,
  description,
  value,
  placeholder = null,
  fact = null,
  onSelect,
  onClear,
}: ISettingsPathFieldProps): ReactElement {
  return (
    <SettingsSection title={label} description={description} fact={fact}>
      <TextField
        fullWidth
        size={"small"}
        placeholder={placeholder ?? "Not selected"}
        value={value ?? ""}
        sx={{ "& .MuiInputBase-input": { fontFamily: "'Cascadia Mono', 'Consolas', monospace", fontSize: "0.75rem" } }}
        slotProps={{
          input: {
            readOnly: true,
            sx: { cursor: "pointer" },
            endAdornment: (
              <Box sx={{ display: "flex", flexShrink: 0 }}>
                {value ? (
                  <Tooltip title={"Clear"}>
                    <IconButton aria-label={"Clear"} onClick={withStoppedPropagation(onClear)}>
                      <ClearIcon fontSize={"small"} />
                    </IconButton>
                  </Tooltip>
                ) : null}

                <Tooltip title={"Choose directory"}>
                  <IconButton aria-label={"Choose directory"} onClick={withStoppedPropagation(onSelect)}>
                    <FolderOpenIcon fontSize={"small"} />
                  </IconButton>
                </Tooltip>
              </Box>
            ),
          },
        }}
        onClick={onSelect}
      />
    </SettingsSection>
  );
}
