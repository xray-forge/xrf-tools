import { default as SearchOffIcon } from "@mui/icons-material/SearchOff";
import { Button, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ICatalogSection } from "@/core/launcher/lib";
import { IApplicationDescriptor } from "@/core/routing/application";
import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ApplicationLauncherCardGrid } from "./ApplicationLauncherCardGrid";
import { ApplicationLauncherRowList } from "./ApplicationLauncherRowList";

export interface IApplicationLauncherCatalogSearch {
  query: string;
  /** Matches found, which exceeds what the sections hold once the search limit applies. */
  matchCount: number;
  onClear: () => void;
}

interface IApplicationLauncherCatalogProps extends BaseComponentProps {
  sections: ReadonlyArray<ICatalogSection>;
  view: TCatalogView;
  /** The search the sections answer, or `null` while the whole catalog is on show. */
  search: Nullable<IApplicationLauncherCatalogSearch>;
  onOpen: (application: IApplicationDescriptor) => void;
}

/**
 * The scrolling body of the launcher.
 */
export function ApplicationLauncherCatalog({
  "data-testid": dataTestId = "application-launcher-catalog",
  id,
  className,
  sections,
  view,
  search,
  onOpen,
}: IApplicationLauncherCatalogProps): ReactElement {
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
            <ApplicationLauncherRowList sections={sections} onOpen={onOpen} />
          ) : (
            <ApplicationLauncherCardGrid sections={sections} onOpen={onOpen} />
          )}
        </>
      )}
    </div>
  );
}
