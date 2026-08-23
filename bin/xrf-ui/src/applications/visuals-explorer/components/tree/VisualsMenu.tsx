import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Box, Typography } from "@mui/material";
import { RichTreeView } from "@mui/x-tree-view";
import { useInjection } from "@wirestate/react";
import { ReactElement, SyntheticEvent, useCallback, useMemo, useState } from "react";

import { VisualTreeItem } from "@/applications/visuals-explorer/components/tree/VisualTreeItem";
import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSearchResults, IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EditorSideMenu } from "@/core/shell/editor/EditorSideMenu";
import {
  getFileItemPath,
  IPathTreeItem,
  LOGICAL_PATH_SEPARATOR,
  parsePathTree,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IVisualsMenuProps extends BaseComponentProps {}

/**
 * Every visual of the browsed world, as a tree.
 */
export function VisualsMenu({
  "data-testid": dataTestId = "visuals-menu",
  id,
  className,
  sx,
}: IVisualsMenuProps = {}): ReactElement {
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);
  const visualsService: VisualsService = useInjection(VisualsService);

  const [expandedItems, setExpandedItems] = useState<Array<string>>([]);

  // Memoized rather than defaulted inline, so an empty listing does not hand the tree a new array every render.
  const visuals: Array<XrayAsset> = useMemo(() => browseService.visuals.value ?? [], [browseService.visuals.value]);

  const items: Array<IPathTreeItem<XrayAsset>> = useMemo(
    () =>
      parsePathTree(
        visuals.map((asset: XrayAsset) => ({ path: asset.logicalPath, payload: asset })),
        LOGICAL_PATH_SEPARATOR
      ),
    [visuals]
  );

  const onOpenPath = useCallback(
    (logicalPath: string) => {
      void visualsService.openAsset(logicalPath, browseService.roots);
    },
    [browseService.roots, visualsService]
  );

  const search: IUseRankedSearch<XrayAsset> = useRankedSearch({
    items: visuals,
    toSearchText: (it: XrayAsset) => it.logicalPath,
    onSelect: (asset: XrayAsset) => onOpenPath(asset.logicalPath),
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<XrayAsset>) => {
        const separatorAt: number = result.item.logicalPath.lastIndexOf("\\");

        return {
          id: result.item.logicalPath,
          label: separatorAt === -1 ? result.item.logicalPath : result.item.logicalPath.slice(separatorAt + 1),
          description: separatorAt === -1 ? undefined : result.item.logicalPath.slice(0, separatorAt),
        };
      }),
    [search.results]
  );

  // Selection follows what is open rather than what was clicked, so a failed open leaves the tree pointing at the model
  // the viewport is explaining.
  const openSource: Nullable<string> = visualsService.visual.value
    ? describeVisualSource(visualsService.visual.value.selected.source)
    : null;
  const selectedItem: Nullable<string> = openSource ? toFileItemId(openSource) : null;

  const onSelectItem = useCallback(
    (_: Nullable<SyntheticEvent>, itemId: Nullable<string>) => {
      const path: Nullable<string> = getFileItemPath(itemId);

      if (path) {
        onOpenPath(path);
      }
    },
    [onOpenPath]
  );

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      header={
        <EditorSearchHeader
          title={"Visuals"}
          count={visuals.length}
          query={search.query}
          placeholder={"Filter visuals"}
          ariaLabel={"Filter visuals"}
          onClear={search.clear}
          onKeyDown={search.onInputKeyDown}
          onQueryChange={search.setQuery}
        />
      }
    >
      {search.isSearching ? (
        <EditorSearchResults
          ariaLabel={"Visual search results"}
          isStale={search.isStale}
          emptyLabel={`No visuals match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          onHoverIndex={search.setActiveIndex}
          onSelect={onOpenPath}
        />
      ) : items.length ? (
        <Box sx={{ padding: 0.5 }}>
          <RichTreeView
            items={items}
            expandedItems={expandedItems}
            selectedItems={selectedItem}
            expansionTrigger={"content"}
            slots={{
              item: VisualTreeItem,
              collapseIcon: FolderOpenIcon,
              expandIcon: FolderIcon,
              endIcon: ViewInArIcon,
            }}
            onExpandedItemsChange={(_, next: Array<string>) => setExpandedItems(next)}
            onSelectedItemsChange={onSelectItem}
          />
        </Box>
      ) : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            {browseService.visuals.isLoading ? "Listing visuals…" : "No visuals found under this root."}
          </Typography>
        </Box>
      )}
    </EditorSideMenu>
  );
}
