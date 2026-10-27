import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { KeyboardEvent, ReactElement, RefObject } from "react";

import { ICatalogGroupFilter } from "@/core/launcher/lib";
import { EToolGroupId } from "@/core/routing/tool";
import { TCatalogView } from "@/core/settings/lib/catalog-view";

import { ToolLauncherGroupFilters } from "./ToolLauncherGroupFilters";
import { ToolLauncherSearchField } from "./ToolLauncherSearchField";
import { ToolLauncherViewToggle } from "./ToolLauncherViewToggle";

interface IToolLauncherHeaderProps extends BaseComponentProps {
  filters: ReadonlyArray<ICatalogGroupFilter>;
  /** Lets the launcher's keyboard shortcut reach the field it does not own. */
  inputRef: RefObject<Nullable<HTMLInputElement>>;
  query: string;
  /** `null` is every group rather than none. */
  selectedGroupId: Nullable<EToolGroupId>;
  totalCount: number;
  /** Counts the catalog below in a line, narrowed the same way the chips narrow it. */
  summary: string;
  view: TCatalogView;
  onClear: () => void;
  onKeyDown: (event: KeyboardEvent<HTMLElement>) => void;
  onQueryChange: (query: string) => void;
  onSelectGroup: (groupId: Nullable<EToolGroupId>) => void;
  onSelectView: (view: TCatalogView) => void;
}

/**
 * Everything above the catalog: what it is, how much of it there is, and the ways to narrow it.
 */
export function ToolLauncherHeader({
  "data-testid": dataTestId = "tool-launcher-header",
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
}: IToolLauncherHeaderProps): ReactElement {
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

        <ToolLauncherSearchField
          inputRef={inputRef}
          query={query}
          onClear={onClear}
          onKeyDown={onKeyDown}
          onQueryChange={onQueryChange}
        />

        <ToolLauncherViewToggle view={view} onSelectView={onSelectView} />
      </div>

      <ToolLauncherGroupFilters
        filters={filters}
        selectedGroupId={selectedGroupId}
        totalCount={totalCount}
        onSelectGroup={onSelectGroup}
      />
    </div>
  );
}
