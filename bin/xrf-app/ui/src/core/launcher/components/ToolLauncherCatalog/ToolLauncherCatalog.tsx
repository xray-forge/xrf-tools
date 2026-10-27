import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { SearchOffIcon } from "@xrf/ui/icons/search-off";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { Button } from "@xrf/ui/material/button";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { ICatalogSection } from "@/core/launcher/lib";
import { IToolDescriptor } from "@/core/routing/tool";
import { TCatalogView } from "@/core/settings/lib/catalog-view";

import { ToolLauncherCardGrid } from "./ToolLauncherCardGrid";
import { ToolLauncherRowList } from "./ToolLauncherRowList";

export interface IToolLauncherCatalogSearch {
  query: string;
  /** Matches found, which exceeds what the sections hold once the search limit applies. */
  matchCount: number;
  onClear: () => void;
}

interface IToolLauncherCatalogProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  view: TCatalogView;
  /** The search the sections answer, or `null` while the whole catalog is on show. */
  search: Nullable<IToolLauncherCatalogSearch>;
  onOpen: (tool: IToolDescriptor) => void;
}

/**
 * The scrolling body of the launcher.
 */
export function ToolLauncherCatalog({
  "data-testid": dataTestId = "tool-launcher-catalog",
  id,
  className,
  sections,
  view,
  search,
  onOpen,
}: IToolLauncherCatalogProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn("min-h-0 grow [scrollbar-gutter:stable] overflow-y-auto px-6 py-5", className)}
    >
      {search && !sections.length ? (
        <EmptyState
          icon={<SearchOffIcon className={"text-content-state-icon text-text-secondary opacity-55"} />}
          title={"No tools match"}
          // Quoted, because an unquoted query reads as part of the sentence carrying it.
          description={`Nothing in the catalog matches "${search.query.trim()}".`}
          action={
            <Button size={"small"} onClick={search.onClear}>
              Clear search
            </Button>
          }
        />
      ) : (
        <>
          {search ? (
            <Typography className={"mb-2 block text-text-secondary"} variant={"caption"}>
              {search.matchCount} {search.matchCount === 1 ? "match" : "matches"}
            </Typography>
          ) : null}

          {view === "rows" ? (
            <ToolLauncherRowList sections={sections} onOpen={onOpen} />
          ) : (
            <ToolLauncherCardGrid sections={sections} onOpen={onOpen} />
          )}
        </>
      )}
    </div>
  );
}
