import { default as SearchOffIcon } from "@mui/icons-material/SearchOff";
import { Box, Button, List, ListSubheader, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { Fragment, ReactElement, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { ApplicationLauncherCard } from "@/core/launcher/ApplicationLauncherCard";
import { ApplicationLauncherHeader, IApplicationLauncherGroupFilter } from "@/core/launcher/ApplicationLauncherHeader";
import { ApplicationLauncherRow } from "@/core/launcher/ApplicationLauncherRow";
import { ApplicationLauncherSection } from "@/core/launcher/ApplicationLauncherSection";
import {
  EApplicationGroupId,
  EApplicationStatus,
  IApplicationDescriptor,
  IApplicationGroup,
} from "@/core/routing/application";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { TCatalogView } from "@/core/settings/lib/catalog-view";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { EmptyState } from "@/core/ui/layout/EmptyState";
import { Nullable } from "@/lib/types/general";

/** One application together with the group it was found under, which search results no longer imply. */
type TCatalogEntry = [IApplicationDescriptor, IApplicationGroup];

interface ILauncherSection {
  group: IApplicationGroup;
  applications: Array<IApplicationDescriptor>;
}

export interface IApplicationLauncherProps {
  applications: ReadonlyArray<IApplicationDescriptor>;
  groups: ReadonlyArray<IApplicationGroup>;
}

/**
 * The searchable home surface for launching applications.
 */
export function ApplicationLauncher({ applications, groups }: IApplicationLauncherProps): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const navigate: NavigateFunction = useNavigate();

  const searchInputRef = useRef<Nullable<HTMLInputElement>>(null);

  const [selectedGroupId, setSelectedGroupId] = useState<Nullable<EApplicationGroupId>>(null);
  const [view, setView] = useState<TCatalogView>(settingsService.catalogView);

  const sections: Array<ILauncherSection> = useMemo(
    () =>
      groups
        .map(
          (group: IApplicationGroup): ILauncherSection => ({
            group,
            applications: applications.filter((application: IApplicationDescriptor) => application.group === group.id),
          })
        )
        .filter((section: ILauncherSection) => section.applications.length > 0),
    [applications, groups]
  );

  const visibleSections: Array<ILauncherSection> = useMemo(
    () =>
      selectedGroupId ? sections.filter((section: ILauncherSection) => section.group.id === selectedGroupId) : sections,
    [sections, selectedGroupId]
  );

  const searchable: Array<TCatalogEntry> = useMemo(
    () =>
      visibleSections.flatMap(({ group, applications: grouped }: ILauncherSection) =>
        grouped.map((application: IApplicationDescriptor): TCatalogEntry => [application, group])
      ),
    [visibleSections]
  );

  const filters: Array<IApplicationLauncherGroupFilter> = useMemo(
    () => sections.map(({ group, applications: grouped }: ILauncherSection) => ({ group, count: grouped.length })),
    [sections]
  );

  const summary: string = useMemo(() => {
    const readyCount: number = searchable.filter(
      ([application]: TCatalogEntry) => application.status === EApplicationStatus.READY
    ).length;

    const parts: Array<string> = [
      `${searchable.length} ${searchable.length === 1 ? "tool" : "tools"}`,
      `${readyCount} ready`,
    ];

    // While one group is chosen this could only ever read "1 group", which its own chip already says.
    if (!selectedGroupId) {
      parts.push(`${sections.length} ${sections.length === 1 ? "group" : "groups"}`);
    }

    return parts.join(" · ");
  }, [searchable, sections.length, selectedGroupId]);

  const isEnabled = useCallback(
    (application: IApplicationDescriptor): boolean =>
      application.status === EApplicationStatus.READY || settingsService.isDevModeEnabled,
    [settingsService]
  );

  const onOpen = useCallback(
    (application: IApplicationDescriptor) => {
      navigate(application.path, { replace: true });
    },
    [navigate]
  );

  const onSelectResult = useCallback(
    ([application]: TCatalogEntry) => {
      if (isEnabled(application)) {
        onOpen(application);
      }
    },
    [isEnabled, onOpen]
  );

  const onSelectView = useCallback(
    (next: TCatalogView) => {
      setView(next);
      settingsService.setCatalogView(next);
    },
    [settingsService]
  );

  const search: IUseRankedSearch<TCatalogEntry> = useRankedSearch({
    items: searchable,
    toSearchText: ([application]: TCatalogEntry) => application.label,
    // The description and the group name match too, so "icons" still finds the six sprite tools whose labels
    // only say "sprite".
    toSecondaryText: ([application, group]: TCatalogEntry) => `${application.description} ${group.label}`,
    onSelect: onSelectResult,
  });

  /** Whether tools are announced under their group at all, which ranked results cannot be. */
  const isGrouped: boolean = !search.isSearching;

  const entries: Array<TCatalogEntry> = search.isSearching
    ? search.results.map(({ item }: ISearchResult<TCatalogEntry>) => item)
    : searchable;

  /** The one place that knows how a run of tools is drawn; everything above only chooses the run. */
  const renderTools = useCallback(
    (tools: Array<TCatalogEntry>): ReactElement =>
      view === "rows" ? (
        <List aria-label={"Tools"} disablePadding={true}>
          {tools.map(([application, group]: TCatalogEntry, index: number) => (
            <Fragment key={application.id}>
              {isGrouped && group.id !== tools[index - 1]?.[1].id ? (
                <ListSubheader
                  disableGutters={true}
                  disableSticky={true}
                  sx={{
                    backgroundColor: "transparent",
                    lineHeight: "unset",
                    paddingX: 1,
                    paddingTop: index === 0 ? 0 : 2,
                    paddingBottom: 0.5,
                  }}
                >
                  <ApplicationLauncherSection
                    group={group}
                    count={tools.filter(([, it]: TCatalogEntry) => it.id === group.id).length}
                  />
                </ListSubheader>
              ) : null}

              <ApplicationLauncherRow
                application={application}
                group={group}
                isEnabled={isEnabled(application)}
                isGroupNamed={!isGrouped}
                onOpen={onOpen}
              />
            </Fragment>
          ))}
        </List>
      ) : (
        <Box
          sx={{
            display: "grid",
            gridTemplateColumns: {
              xs: "repeat(1, minmax(0, 1fr))",
              sm: "repeat(2, minmax(0, 1fr))",
              md: "repeat(3, minmax(0, 1fr))",
              lg: "repeat(3, minmax(0, 1fr))",
              xl: "repeat(4, minmax(0, 1fr))",
            },
            gap: 1.5,
          }}
        >
          {tools.map(([application, group]: TCatalogEntry) => (
            <ApplicationLauncherCard
              key={application.id}
              application={application}
              group={group}
              isEnabled={isEnabled(application)}
              // A card names its group exactly where no heading above it does.
              isGroupNamed={!isGrouped}
              onOpen={onOpen}
            />
          ))}
        </Box>
      ),
    [isEnabled, isGrouped, onOpen, view]
  );

  useEffect(() => {
    function onWindowKeyDown(event: KeyboardEvent): void {
      const isEditing: boolean =
        event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement;

      if (((event.ctrlKey || event.metaKey) && event.key === "k") || (event.key === "/" && !isEditing)) {
        event.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
      }
    }

    window.addEventListener("keydown", onWindowKeyDown);

    return () => window.removeEventListener("keydown", onWindowKeyDown);
  }, []);

  return (
    <EditorLayout toolbar={<EditorToolbar />}>
      <Box sx={{ display: "flex", flexDirection: "column", width: "100%", height: "100%", minHeight: 0 }}>
        <Box sx={{ flexShrink: 0, paddingX: 3, paddingTop: 3 }}>
          <Box>
            <ApplicationLauncherHeader
              filters={filters}
              inputRef={searchInputRef}
              query={search.query}
              selectedGroupId={selectedGroupId}
              summary={summary}
              totalCount={applications.length}
              view={view}
              onClear={search.clear}
              onKeyDown={search.onInputKeyDown}
              onQueryChange={search.setQuery}
              onSelectGroup={setSelectedGroupId}
              onSelectView={onSelectView}
            />
          </Box>
        </Box>

        <Box
          data-testid={"launcher-catalog"}
          sx={{
            flexGrow: 1,
            minHeight: 0,
            scrollbarGutter: "stable",
            overflowY: "auto",
            paddingX: 3,
            paddingY: 2.5,
          }}
        >
          <Box>
            {search.isSearching ? (
              search.results.length ? (
                // Dimmed while a newer keystroke is still being filtered, so stale rows do not read as final.
                <Box sx={{ opacity: search.isStale ? 0.6 : 1, transition: "opacity 120ms ease" }}>
                  <Typography variant={"caption"} sx={{ display: "block", marginBottom: 1, color: "text.secondary" }}>
                    {search.total} {search.total === 1 ? "match" : "matches"}
                  </Typography>

                  {renderTools(entries)}
                </Box>
              ) : (
                <EmptyState
                  icon={<SearchOffIcon sx={{ fontSize: 40, color: "text.secondary", opacity: 0.55 }} />}
                  title={"No tools match"}
                  // Quoted, because an unquoted query reads as part of the sentence carrying it.
                  description={`Nothing in the catalog matches "${search.query.trim()}".`}
                  action={
                    <Button size={"small"} onClick={search.clear}>
                      Clear search
                    </Button>
                  }
                />
              )
            ) : view === "grid" ? (
              <Box sx={{ display: "flex", flexDirection: "column", gap: 4 }}>
                {visibleSections.map(({ group, applications: grouped }: ILauncherSection) => (
                  <Box key={group.id} sx={{ display: "flex", flexDirection: "column", gap: 1 }}>
                    <ApplicationLauncherSection group={group} count={grouped.length} />

                    {renderTools(
                      grouped.map((application: IApplicationDescriptor): TCatalogEntry => [application, group])
                    )}
                  </Box>
                ))}
              </Box>
            ) : (
              renderTools(entries)
            )}
          </Box>
        </Box>
      </Box>
    </EditorLayout>
  );
}
