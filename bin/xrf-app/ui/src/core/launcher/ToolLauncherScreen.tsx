import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement, useCallback, useMemo, useRef } from "react";
import { NavigateFunction, useNavigate } from "react-router-dom";

import { PRODUCT_NAME } from "@/core/configs";
import { IToolDescriptor, IToolGroup } from "@/core/routing/tool";
import { useSearchFocusTarget } from "@/core/search/lib";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorToolbar } from "@/core/shell/chrome/EditorToolbar";

import { IToolLauncherCatalogSearch, ToolLauncherCatalog } from "./components/ToolLauncherCatalog";
import { ToolLauncherHeader } from "./components/ToolLauncherHeader";
import { ICatalogEntry, IUseToolCatalog, useToolCatalog } from "./lib";

export interface IToolLauncherProps extends BaseComponentProps {
  tools: ReadonlyArray<IToolDescriptor>;
  groups: ReadonlyArray<IToolGroup>;
}

/**
 * Draws the catalog, inside the container its services live in.
 */
export function ToolLauncherScreen({
  "data-testid": dataTestId = "tool-launcher",
  id,
  className,
  tools,
  groups,
}: IToolLauncherProps): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const navigate: NavigateFunction = useNavigate();

  const searchInputRef = useRef<Nullable<HTMLInputElement>>(null);

  const onOpen = useCallback(
    (tool: IToolDescriptor) => {
      navigate(tool.path, { replace: true });
    },
    [navigate]
  );

  const onSelectResult = useCallback(({ tool }: ICatalogEntry) => onOpen(tool), [onOpen]);

  const catalog: IUseToolCatalog = useToolCatalog({
    tools,
    groups,
    isDevModeEnabled: settingsService.isDevModeEnabled,
    onSelect: onSelectResult,
  });

  const search: Nullable<IToolLauncherCatalogSearch> = useMemo(
    () =>
      catalog.search.isSearching
        ? { query: catalog.search.query, matchCount: catalog.search.total, onClear: catalog.search.clear }
        : null,
    [catalog.search.isSearching, catalog.search.query, catalog.search.total, catalog.search.clear]
  );

  useSearchFocusTarget(searchInputRef);

  return (
    <EditorLayout
      data-testid={dataTestId}
      id={id}
      className={className}
      toolbar={<EditorToolbar title={PRODUCT_NAME} />}
    >
      <div className={"flex h-full min-h-0 w-full flex-col"}>
        <div className={"shrink-0 px-6 pt-6"}>
          <ToolLauncherHeader
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

        <ToolLauncherCatalog
          sections={catalog.sections}
          view={settingsService.catalogView}
          search={search}
          onOpen={onOpen}
        />
      </div>
    </EditorLayout>
  );
}
