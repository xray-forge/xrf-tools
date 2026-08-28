import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Box, Tooltip, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useMemo } from "react";

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
  splitLogicalPath,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
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
 * @param item - Node being labelled; only a leaf carries the asset that could have come from a volume.
 * @returns The decorated label, or the plain name for a directory or a loose file.
 */
function renderVisualLabel(item: ITreeNode<XrayAsset>): ReactNode {
  if (item.payload?.container.kind !== "archive") {
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

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

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
      // Selection is written from what was asked for, never derived from what the viewport ended up holding: a
      // model that fails to load leaves its row selected, which is the only way back to retrying it.
      reveal(toFileItemId(logicalPath));

      void visualsService.openAsset(logicalPath, browseService.rootPaths);
    },
    [browseService.rootPaths, reveal, visualsService]
  );

  const search: IUseRankedSearch<XrayAsset> = useRankedSearch({
    items: visuals,
    toSearchText: (it: XrayAsset) => it.logicalPath,
    onSelect: (asset: XrayAsset) => onOpenPath(asset.logicalPath),
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<XrayAsset>) => {
        const { name, directory } = splitLogicalPath(result.item.logicalPath);

        return { id: result.item.logicalPath, label: name, description: directory ?? undefined };
      }),
    [search.results]
  );

  const onSelectAsset = useCallback((item: ITreeNode<XrayAsset>) => tree.select(item.id), [tree]);

  const onActivateAsset = useCallback(
    (item: ITreeNode<XrayAsset>) => {
      const path: Nullable<string> = getFileItemPath(item.id);

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
        <VirtualizedTree<XrayAsset>
          ariaLabel={"Visuals"}
          icons={VISUAL_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          renderLabel={renderVisualLabel}
          onSelect={onSelectAsset}
          onActivate={onActivateAsset}
          onToggleExpanded={tree.toggleExpanded}
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
