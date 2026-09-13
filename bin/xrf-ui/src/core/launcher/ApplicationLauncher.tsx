import { Box } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useRef } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { IApplicationDescriptor, IApplicationGroup } from "@/core/routing/application";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { EditorToolbar } from "@/core/shell/editor/EditorToolbar";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

import { ApplicationLauncherCatalog } from "./components/ApplicationLauncherCatalog";
import { ApplicationLauncherHeader } from "./components/ApplicationLauncherHeader";
import { ICatalogEntry, IUseApplicationCatalog, useApplicationCatalog, useSearchHotkey } from "./lib";

interface IApplicationLauncherProps extends BaseComponentProps {
  applications: ReadonlyArray<IApplicationDescriptor>;
  groups: ReadonlyArray<IApplicationGroup>;
}

/**
 * The searchable home surface for launching applications.
 */
export function ApplicationLauncher({
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

  const catalog: IUseApplicationCatalog = useApplicationCatalog({ applications, groups, onSelect: onSelectResult });

  useSearchHotkey(searchInputRef);

  return (
    <EditorLayout data-testid={dataTestId} id={id} className={className} toolbar={<EditorToolbar />}>
      <Box sx={{ display: "flex", flexDirection: "column", width: "100%", height: "100%", minHeight: 0 }}>
        <Box sx={{ flexShrink: 0, paddingX: 3, paddingTop: 3 }}>
          <ApplicationLauncherHeader
            filters={catalog.filters}
            inputRef={searchInputRef}
            query={catalog.search.query}
            selectedGroupId={catalog.selectedGroupId}
            summary={catalog.summary}
            totalCount={applications.length}
            view={settingsService.catalogView}
            onClear={catalog.search.clear}
            onKeyDown={catalog.search.onInputKeyDown}
            onQueryChange={catalog.search.setQuery}
            onSelectGroup={catalog.onSelectGroup}
            onSelectView={settingsService.setCatalogView}
          />
        </Box>

        <ApplicationLauncherCatalog
          sections={catalog.sections}
          view={settingsService.catalogView}
          search={
            catalog.search.isSearching
              ? {
                  query: catalog.search.query,
                  matchCount: catalog.search.total,
                  onClear: catalog.search.clear,
                }
              : null
          }
          onOpen={onOpen}
        />
      </Box>
    </EditorLayout>
  );
}
