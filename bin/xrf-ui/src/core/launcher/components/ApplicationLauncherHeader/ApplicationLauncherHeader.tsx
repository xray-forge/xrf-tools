import { Box, Typography } from "@mui/material";
import { KeyboardEvent, ReactElement, RefObject } from "react";

import { ICatalogGroupFilter } from "@/core/launcher/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ApplicationLauncherGroupFilters } from "./ApplicationLauncherGroupFilters";
import { ApplicationLauncherSearchField } from "./ApplicationLauncherSearchField";
import { ApplicationLauncherViewToggle } from "./ApplicationLauncherViewToggle";

interface IApplicationLauncherHeaderProps extends BaseComponentProps {
  filters: ReadonlyArray<ICatalogGroupFilter>;
  /** Lets the launcher's keyboard shortcut reach the field it does not own. */
  inputRef: RefObject<Nullable<HTMLInputElement>>;
  query: string;
  /** `null` is every group rather than none. */
  selectedGroupId: Nullable<EApplicationGroupId>;
  totalCount: number;
  /** Counts the catalog below in a line, narrowed the same way the chips narrow it. */
  summary: string;
  view: TCatalogView;
  onClear: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
  onSelectGroup: (groupId: Nullable<EApplicationGroupId>) => void;
  onSelectView: (view: TCatalogView) => void;
}

/**
 * Everything above the catalog: what it is, how much of it there is, and the ways to narrow it.
 */
export function ApplicationLauncherHeader({
  "data-testid": dataTestId = "application-launcher-header",
  id,
  className,
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
      data-testid={dataTestId}
      id={id}
      className={className}
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

        <ApplicationLauncherSearchField
          inputRef={inputRef}
          query={query}
          onClear={onClear}
          onKeyDown={onKeyDown}
          onQueryChange={onQueryChange}
        />

        <ApplicationLauncherViewToggle view={view} onSelectView={onSelectView} />
      </Box>

      <ApplicationLauncherGroupFilters
        filters={filters}
        selectedGroupId={selectedGroupId}
        totalCount={totalCount}
        onSelectGroup={onSelectGroup}
      />
    </Box>
  );
}
