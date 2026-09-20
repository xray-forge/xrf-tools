import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo, useRef } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { useSearchFocusTarget } from "@/core/search/lib";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ApplicationLauncherCatalog, IApplicationLauncherCatalogSearch } from "./components/ApplicationLauncherCatalog";
import { ApplicationLauncherHeader } from "./components/ApplicationLauncherHeader";
import { ICatalogEntry, IUseApplicationCatalog, useApplicationCatalog } from "./lib";

export interface IApplicationLauncherProps extends BaseComponentProps {
  applications: ReadonlyArray<IApplicationDescriptor>;
  groups: ReadonlyArray<IApplicationGroup>;
}

/**
 * Draws the catalog, inside the container its services live in.
 */
export function ApplicationLauncherScreen({
  "data-testid": dataTestId = "application-launcher",
  id,
  className,
  applications,
  groups,
}: IApplicationLauncherProps): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const navigate: NavigateFunction = useNavigate();

  const searchInputRef = useRef<Nullable<HTMLInputElement>>(null);

  const onOpen = useCallback(
    (application: IApplicationDescriptor) => {
      navigate(application.path, { replace: true });
    },
    [navigate]
  );

  const onSelectResult = useCallback(({ application }: ICatalogEntry) => onOpen(application), [onOpen]);

  const catalog: IUseApplicationCatalog = useApplicationCatalog({
    applications,
    groups,
    isDevModeEnabled: settingsService.isDevModeEnabled,
    onSelect: onSelectResult,
  });

  const search: Nullable<IApplicationLauncherCatalogSearch> = useMemo(
    () =>
      catalog.search.isSearching
        ? { query: catalog.search.query, matchCount: catalog.search.total, onClear: catalog.search.clear }
        : null,
    [catalog.search.isSearching, catalog.search.query, catalog.search.total, catalog.search.clear]
  );

  useSearchFocusTarget(searchInputRef);

  return (
    <EditorLayout data-testid={dataTestId} id={id} className={className} toolbar={<EditorToolbar />}>
      <div className={"flex h-full min-h-0 w-full flex-col"}>
        <div className={"shrink-0 px-6 pt-6"}>
          <ApplicationLauncherHeader
            filters={catalog.filters}
            inputRef={searchInputRef}
            query={catalog.search.query}
            selectedGroupId={catalog.selectedGroupId}
            summary={catalog.summary}
            totalCount={catalog.totalCount}
            view={settingsService.catalogView}
            onClear={catalog.search.clear}
            onKeyDown={catalog.search.onInputKeyDown}
            onQueryChange={catalog.search.setQuery}
            onSelectGroup={catalog.onSelectGroup}
            onSelectView={settingsService.setCatalogView}
          />
        </div>

        <ApplicationLauncherCatalog
          sections={catalog.sections}
          view={settingsService.catalogView}
          search={search}
          onOpen={onOpen}
        />
      </div>
    </EditorLayout>
  );
}
