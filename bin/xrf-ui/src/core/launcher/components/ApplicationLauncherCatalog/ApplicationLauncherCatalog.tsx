import { default as SearchOffIcon } from "@mui/icons-material/SearchOff";
import { Box, Button, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ICatalogSection } from "@/core/launcher/lib";
import { IApplicationDescriptor } from "@/core/routing/application";
import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { EmptyState } from "@/core/ui/layout/EmptyState";
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
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{
        flexGrow: 1,
        minHeight: 0,
        scrollbarGutter: "stable",
        overflowY: "auto",
        paddingX: 3,
        paddingY: 2.5,
      }}
    >
      {search && !sections.length ? (
        <EmptyState
          icon={<SearchOffIcon sx={{ fontSize: 40, color: "text.secondary", opacity: 0.55 }} />}
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
            <Typography variant={"caption"} sx={{ display: "block", marginBottom: 1, color: "text.secondary" }}>
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
    </Box>
  );
}
