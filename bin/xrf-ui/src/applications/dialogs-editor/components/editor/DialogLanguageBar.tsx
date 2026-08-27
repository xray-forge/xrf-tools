import { Box, MenuItem, TextField } from "@mui/material";
import { ReactElement } from "react";

import { Nullable } from "@/lib/types/general";

export interface IDialogLanguageBarProps {
  languages: ReadonlyArray<string>;
  /** The language actually resolved, which the backend echoes back rather than the one asked for. */
  selected: Nullable<string>;
  onSelect: (language: string) => void;
}

/**
 * Which language the phrase lines are read in.
 *
 * One selector, not the reference-and-target pair the translations editor needs: this surface shows
 * text rather than translating it, so there is nothing to compare against.
 *
 * Renders nothing when the project read no text tree. A switcher over no languages offers a choice
 * that cannot be made, and the editor already says "no text" in its status bar.
 */
export function DialogLanguageBar({ languages, selected, onSelect }: IDialogLanguageBarProps): ReactElement | null {
  if (!languages.length) {
    return null;
  }

  return (
    <Box sx={{ borderBottom: "1px solid", borderColor: "divider", padding: 1 }}>
      <TextField
        select
        size={"small"}
        label={"Language"}
        sx={{ minWidth: 180 }}
        value={selected ?? ""}
        onChange={(event) => onSelect(event.target.value)}
      >
        {languages.map((it: string) => (
          <MenuItem key={it} value={it}>
            {it}
          </MenuItem>
        ))}
      </TextField>
    </Box>
  );
}
