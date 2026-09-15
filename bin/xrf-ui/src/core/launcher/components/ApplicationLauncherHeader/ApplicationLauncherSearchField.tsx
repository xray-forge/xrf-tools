import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as SearchIcon } from "@mui/icons-material/Search";
import { Box, IconButton, InputAdornment, TextField, Tooltip } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, KeyboardEvent, ReactElement, RefObject, useCallback } from "react";

import { formatChord, parseChord } from "@/core/keybinds";
import { KeymapService } from "@/core/keybinds/services/keymap";
import { FOCUS_SEARCH_KEYBIND_COMMAND } from "@/core/search/commands";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable, Optional } from "@/lib/types/general";

interface IApplicationLauncherSearchFieldProps extends BaseComponentProps {
  /** Lets the launcher's keyboard shortcut reach the field it does not own. */
  inputRef: RefObject<Nullable<HTMLInputElement>>;
  query: string;
  onClear: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
}

/**
 * The catalog's filter-as-you-type field.
 */
export function ApplicationLauncherSearchField({
  "data-testid": dataTestId = "application-launcher-search-field",
  id,
  className,
  inputRef,
  query,
  onClear,
  onKeyDown,
  onQueryChange,
}: IApplicationLauncherSearchFieldProps): ReactElement {
  const keymapService: KeymapService = useInjection(KeymapService);
  // Read rather than written down: the overlay can rebind the command, and a hint nobody updates is a hint that lies.
  const chord: Optional<string> = keymapService.getReachableChords(FOCUS_SEARCH_KEYBIND_COMMAND).at(0);

  const onFieldKeyDown = useCallback(
    (event: KeyboardEvent<HTMLElement>) => {
      if (event.key === "Escape") {
        event.preventDefault();
        inputRef.current?.blur();

        return;
      }

      onKeyDown(event);
    },
    [inputRef, onKeyDown]
  );

  const onChange = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      onQueryChange(event.target.value);
    },
    [onQueryChange]
  );

  return (
    <TextField
      data-testid={dataTestId}
      id={id}
      className={className}
      value={query}
      placeholder={"Search tools"}
      inputRef={inputRef}
      sx={{ width: 320, flexShrink: 1, minWidth: 180 }}
      slotProps={{
        htmlInput: {
          "aria-label": "Search tools",
        },
        input: {
          startAdornment: (
            <InputAdornment position={"start"}>
              <SearchIcon fontSize={"small"} />
            </InputAdornment>
          ),
          // The shortcut hint stands where the clear button will be, so neither moves the field's width.
          endAdornment: (
            <InputAdornment position={"end"}>
              {query ? (
                <Tooltip title={"Clear search"}>
                  <IconButton aria-label={"Clear tool search"} edge={"end"} onClick={onClear}>
                    <ClearIcon fontSize={"small"} />
                  </IconButton>
                </Tooltip>
              ) : chord ? (
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
                  {formatChord(parseChord(chord))}
                </Box>
              ) : null}
            </InputAdornment>
          ),
        },
      }}
      onKeyDown={onFieldKeyDown}
      onChange={onChange}
    />
  );
}
