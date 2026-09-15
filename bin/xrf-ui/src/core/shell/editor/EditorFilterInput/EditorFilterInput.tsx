import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as SearchIcon } from "@mui/icons-material/Search";
import { Box, IconButton, InputAdornment, TextField, Tooltip } from "@mui/material";
import { useForkRef } from "@mui/material/utils";
import { KeyboardEvent, ReactElement, Ref, RefObject, useRef } from "react";

import { StyledComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IEditorFilterInputProps extends StyledComponentProps {
  ariaLabel: string;
  /** Lets an owner reach the input it does not render, to hand it the caret. */
  inputRef?: RefObject<Nullable<HTMLInputElement>>;
  query: string;
  placeholder: string;
  /** Chord that reaches this field, shown while it is empty. Omitted where no chord does. */
  chord?: string;
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
  ariaLabel,
  inputRef,
  query,
  placeholder,
  chord,
  onClear,
  onKeyDown,
  onQueryChange,
  sx,
}: IEditorFilterInputProps): ReactElement {
  const ownRef = useRef<Nullable<HTMLInputElement>>(null);
  const handleRef: Ref<HTMLInputElement> = useForkRef(ownRef, inputRef);

  function clear(): void {
    if (onClear) {
      onClear();
    } else {
      onQueryChange("");
    }

    ownRef.current?.focus();
  }

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      inputRef={handleRef}
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
          // The hint stands where the clear button will be, so neither moves the field's width.
          endAdornment: query ? (
            <InputAdornment position={"end"}>
              <Tooltip title={"Clear filter"}>
                <IconButton aria-label={"Clear filter"} edge={"end"} onClick={clear}>
                  <ClearIcon fontSize={"small"} />
                </IconButton>
              </Tooltip>
            </InputAdornment>
          ) : chord ? (
            <InputAdornment position={"end"}>
              <Box
                aria-hidden={true}
                sx={{
                  paddingX: 0.5,
                  color: "text.secondary",
                  border: "1px solid",
                  borderColor: "divider",
                  borderRadius: 1,
                  fontSize: "0.625rem",
                  lineHeight: "16px",
                  whiteSpace: "nowrap",
                }}
              >
                {chord}
              </Box>
            </InputAdornment>
          ) : null,
        },
      }}
      onChange={(event) => onQueryChange(event.target.value)}
    />
  );
}
