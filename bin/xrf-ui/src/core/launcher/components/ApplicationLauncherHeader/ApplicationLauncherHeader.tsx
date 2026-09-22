import { Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { KeyboardEvent, ReactElement, RefObject } from "react";

import { ICatalogGroupFilter } from "@/core/launcher/lib";
import { EApplicationGroupId } from "@/core/routing/application";
import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

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
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("flex shrink-0 flex-col gap-3 border-b border-divider pb-3", className)}
    >
      <div className={"flex items-center gap-4"}>
        <div className={"flex min-w-0 items-baseline gap-2"}>
          <Typography component={"h1"} variant={"h5"}>
            Tools
          </Typography>

          <Typography
            variant={"body2"}
            className={"overflow-hidden text-ellipsis whitespace-nowrap text-text-secondary"}
          >
            {summary}
          </Typography>
        </div>

        <div className={"grow"} />

        <ApplicationLauncherSearchField
          inputRef={inputRef}
          query={query}
          onClear={onClear}
          onKeyDown={onKeyDown}
          onQueryChange={onQueryChange}
        />

        <ApplicationLauncherViewToggle view={view} onSelectView={onSelectView} />
      </div>

      <ApplicationLauncherGroupFilters
        filters={filters}
        selectedGroupId={selectedGroupId}
        totalCount={totalCount}
        onSelectGroup={onSelectGroup}
      />
    </div>
  );
}
