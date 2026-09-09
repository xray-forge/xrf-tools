import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as SearchIcon } from "@mui/icons-material/Search";
import { IconButton, InputAdornment, TextField, Tooltip } from "@mui/material";
import { KeyboardEvent, ReactElement, useRef } from "react";

import { StyledComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IEditorFilterInputProps extends StyledComponentProps {
  query: string;
  placeholder: string;
  ariaLabel: string;
  /** Clears owner-specific search state; defaults to requesting an empty query. */
  onClear?: () => void;
  /** Lets the input drive a result list without handling keys from the clear button. */
  onKeyDown?: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
}

/** A controlled editor filter that returns focus to its input after clearing. */
export function EditorFilterInput({
  "data-testid": dataTestId = "editor-filter-input",
  id,
  className,
  query,
  placeholder,
  ariaLabel,
  onClear,
  onKeyDown,
  onQueryChange,
  sx,
}: IEditorFilterInputProps): ReactElement {
  const inputRef = useRef<Nullable<HTMLInputElement>>(null);

  function clear(): void {
    if (onClear) {
      onClear();
    } else {
      onQueryChange("");
    }

    inputRef.current?.focus();
  }

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      inputRef={inputRef}
      fullWidth
      size={"small"}
      value={query}
      placeholder={placeholder}
      slotProps={{
        htmlInput: { "aria-label": ariaLabel, onKeyDown },
        input: {
          startAdornment: (
            <InputAdornment position={"start"}>
              <SearchIcon fontSize={"small"} />
            </InputAdornment>
          ),
          endAdornment: query ? (
            <InputAdornment position={"end"}>
              <Tooltip title={"Clear filter"}>
                <IconButton aria-label={"Clear filter"} edge={"end"} onClick={clear}>
                  <ClearIcon fontSize={"small"} />
                </IconButton>
              </Tooltip>
            </InputAdornment>
          ) : null,
        },
      }}
      onChange={(event) => onQueryChange(event.target.value)}
    />
  );
}
