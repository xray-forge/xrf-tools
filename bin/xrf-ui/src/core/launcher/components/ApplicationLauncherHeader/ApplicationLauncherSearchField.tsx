import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as SearchIcon } from "@mui/icons-material/Search";
import { Box, IconButton, InputAdornment, TextField, Tooltip } from "@mui/material";
import { ChangeEvent, KeyboardEvent, ReactElement, RefObject } from "react";

import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
              ) : (
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
                  Ctrl K
                </Box>
              )}
            </InputAdornment>
          ),
        },
      }}
      onKeyDown={onKeyDown}
      onChange={(event: ChangeEvent<HTMLInputElement>) => onQueryChange(event.target.value)}
    />
  );
}
