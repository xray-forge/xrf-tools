import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as SearchIcon } from "@mui/icons-material/Search";
import { IconButton, InputAdornment, TextField, Tooltip } from "@mui/material";
import { useForkRef } from "@mui/material/utils";
import { Nullable } from "@xrf/types";
import { KeyboardEvent, ReactElement, Ref, RefObject, useRef } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IEditorFilterInputProps extends BaseComponentProps {
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
              <div
                aria-hidden={true}
                className={
                  "rounded-surface border border-divider px-1 text-badge leading-4 whitespace-nowrap text-text-secondary"
                }
              >
                {chord}
              </div>
            </InputAdornment>
          ) : null,
        },
      }}
      onChange={(event) => onQueryChange(event.target.value)}
    />
  );
}
