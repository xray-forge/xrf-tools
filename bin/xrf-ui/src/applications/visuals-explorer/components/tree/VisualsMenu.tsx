import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Box, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useMemo, useState } from "react";

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
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
const VISUAL_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <ViewInArIcon />,
};

/**
 * Marks a visual that was read out of an archive, which is all a row says beyond its name.
 *
 * Replaces the former `VisualTreeItem` slot: with a flat tree the payload is in hand at render time, so
 * the marker no longer needs a component that looks the item back up by id.
 *
 * @param item - Leaf being labelled.
 * @returns The decorated label, or the plain name for a loose file.
 */
function renderVisualLabel(item: IPathTreeItem<XrayAsset>): ReactNode {
  if (item.kind !== "file" || item.payload.container.kind !== "archive") {
    return item.label;
  }

  return (
    <Box sx={{ alignItems: "center", display: "flex", gap: 0.75, minWidth: 0 }}>
      <Box component={"span"} sx={{ minWidth: 0, overflow: "hidden", textOverflow: "ellipsis" }}>
        {item.label}
      </Box>

      <Tooltip title={"Read from an archive volume"}>
        <Typography
          component={"span"}
          variant={"caption"}
          sx={{ color: "text.secondary", flexShrink: 0, opacity: 0.75 }}
        >
          db
        </Typography>
      </Tooltip>
    </Box>
  );
}

/**
 * Every visual of the browsed roots, as a tree.
 */
export function VisualsMenu({
  "data-testid": dataTestId = "visuals-menu",
  id,
  className,
  sx,
}: BaseComponentProps = {}): ReactElement {
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);
  const visualsService: VisualsService = useInjection(VisualsService);

  const [expandedIds, setExpandedIds] = useState<ReadonlySet<string>>(() => new Set());

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
      void visualsService.openAsset(logicalPath, browseService.rootPaths);
    },
    [browseService.rootPaths, visualsService]
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

  const onSelectAsset = useCallback(
    (item: IPathTreeItem<XrayAsset>) => {
      const path: Nullable<string> = getFileItemPath(item.id);

      if (path) {
        onOpenPath(path);
      }
    },
    [onOpenPath]
  );

  const onToggleExpanded = useCallback((itemId: string) => {
    setExpandedIds((current: ReadonlySet<string>) => {
      const next: Set<string> = new Set(current);

      if (!next.delete(itemId)) {
        next.add(itemId);
      }

      return next;
    });
  }, []);

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
        <VirtualizedTree<XrayAsset>
          ariaLabel={"Visuals"}
          icons={VISUAL_TREE_ICONS}
          items={items}
          expandedIds={expandedIds}
          selectedId={selectedItem}
          renderLabel={renderVisualLabel}
          onSelect={onSelectAsset}
          onToggleExpanded={onToggleExpanded}
        />
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
