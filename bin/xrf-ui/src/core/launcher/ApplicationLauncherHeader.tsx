import { default as ClearIcon } from "@mui/icons-material/Clear";
import { default as GridViewIcon } from "@mui/icons-material/GridView";
import { default as SearchIcon } from "@mui/icons-material/Search";
import { default as TableRowsIcon } from "@mui/icons-material/TableRows";
import {
  Box,
  Chip,
  IconButton,
  InputAdornment,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Tooltip,
  Typography,
} from "@mui/material";
import { ChangeEvent, KeyboardEvent, ReactElement, RefObject } from "react";

import { EApplicationGroupId, IApplicationGroup } from "@/core/routing/application";
import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { Nullable } from "@/lib/types/general";

export interface IApplicationLauncherGroupFilter {
  group: IApplicationGroup;
  count: number;
}

export interface IApplicationLauncherHeaderProps {
  filters: ReadonlyArray<IApplicationLauncherGroupFilter>;
  /** Lets the launcher's keyboard shortcut reach the field it does not own. */
  inputRef: RefObject<Nullable<HTMLInputElement>>;
  query: string;
  /** `null` is every group rather than none. */
  selectedGroupId: Nullable<EApplicationGroupId>;
  totalCount: number;
  /** Counts the catalog below in a line, narrowed the same way the chips narrow it. */
  summary: string;
  /** How the body below draws its tools; the filters themselves are the same either way. */
  view: TCatalogView;
  onClear: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
  onSelectGroup: (groupId: Nullable<EApplicationGroupId>) => void;
  onSelectView: (view: TCatalogView) => void;
}

/**
 * Everything above the catalog: what it is, how much of it there is, and the two ways to narrow it.
 */
export function ApplicationLauncherHeader({
  filters,
  inputRef,
  query,
  selectedGroupId,
  totalCount,
  summary,
  view,
  onClear,
  onKeyDown,
  onQueryChange,
  onSelectGroup,
  onSelectView,
}: IApplicationLauncherHeaderProps): ReactElement {
  return (
    <Box
      sx={{
        display: "flex",
        flexDirection: "column",
        flexShrink: 0,
        gap: 1.5,
        paddingBottom: 1.5,
        borderBottom: 1,
        borderColor: "divider",
      }}
    >
      <Box sx={{ display: "flex", alignItems: "center", gap: 2 }}>
        <Box sx={{ display: "flex", alignItems: "baseline", gap: 1, minWidth: 0 }}>
          <Typography component={"h1"} variant={"h5"}>
            Tools
          </Typography>

          <Typography
            variant={"body2"}
            sx={{ color: "text.secondary", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
          >
            {summary}
          </Typography>
        </Box>

        <Box sx={{ flexGrow: 1 }} />

        <TextField
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
              endAdornment: query ? (
                <InputAdornment position={"end"}>
                  <Tooltip title={"Clear search"}>
                    <IconButton aria-label={"Clear tool search"} edge={"end"} onClick={onClear}>
                      <ClearIcon fontSize={"small"} />
                    </IconButton>
                  </Tooltip>
                </InputAdornment>
              ) : (
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
                    Ctrl K
                  </Box>
                </InputAdornment>
              ),
            },
          }}
          onKeyDown={onKeyDown}
          onChange={(event: ChangeEvent<HTMLInputElement>) => onQueryChange(event.target.value)}
        />

        <ToggleButtonGroup
          aria-label={"Catalog view"}
          exclusive={true}
          size={"small"}
          value={view}
          // Releasing the active button would leave the catalog with no view at all.
          onChange={(_, next: Nullable<TCatalogView>) => next && onSelectView(next)}
        >
          <Tooltip title={"Grid view"}>
            <ToggleButton aria-label={"Grid view"} value={"grid"}>
              <GridViewIcon fontSize={"small"} />
            </ToggleButton>
          </Tooltip>

          <Tooltip title={"Row view"}>
            <ToggleButton aria-label={"Row view"} value={"rows"}>
              <TableRowsIcon fontSize={"small"} />
            </ToggleButton>
          </Tooltip>
        </ToggleButtonGroup>
      </Box>

      <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.75 }}>
        <Chip
          aria-pressed={selectedGroupId === null}
          size={"small"}
          label={`All ${totalCount}`}
          color={selectedGroupId === null ? "primary" : "default"}
          variant={selectedGroupId === null ? "filled" : "outlined"}
          onClick={() => onSelectGroup(null)}
        />

        {filters.map(({ group, count }: IApplicationLauncherGroupFilter) => {
          const isSelected: boolean = selectedGroupId === group.id;

          return (
            <Chip
              key={group.id}
              size={"small"}
              label={`${group.label} ${count}`}
              aria-pressed={isSelected}
              color={isSelected ? "primary" : "default"}
              variant={isSelected ? "filled" : "outlined"}
              // Clicking the active chip is how someone gets back to everything without aiming at `All`.
              onClick={() => onSelectGroup(isSelected ? null : group.id)}
            />
          );
        })}
      </Box>
    </Box>
  );
}
