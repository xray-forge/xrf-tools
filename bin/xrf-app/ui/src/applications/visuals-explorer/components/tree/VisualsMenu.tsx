import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { AssetSource, EAssetSource } from "@xrf/ipc/types/xrf-app-core";
import { XrayAsset } from "@xrf/ipc/types/xrf-vfs";
import { LOGICAL_PATH_SEPARATOR } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FolderIcon, FolderOpenIcon, ViewInArIcon } from "@xrf/ui/icons";
import { EmptyListing } from "@xrf/ui/layout";
import { IEditorSearchResultRow } from "@xrf/ui/shell/editor/EditorSearchResults";
import { IPathTreeItem, parsePathTree, toFileItemId } from "@xrf/ui/tree/path-tree";
import { ITreeNode } from "@xrf/ui/tree/tree-node";
import { ARCHIVED_CAPTION, TreeRowLabel } from "@xrf/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@xrf/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@xrf/ui/tree/VirtualizedTree";
import { ReactElement, useCallback, useMemo } from "react";

import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { VisualsBrowseService } from "@/applications/visuals-explorer/services/visuals-browse";
import { isLooseContainer } from "@/core/assets/lib/container";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { toPathSearchRow } from "@/core/shell/editor/EditorSearchMenu/path-search-row";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
const VISUAL_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <ViewInArIcon />,
};

/**
 * Every visual of the browsed roots, as a tree.
 */
export function VisualsMenu({
  "data-testid": dataTestId = "visuals-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const browseService: VisualsBrowseService = useInjection(VisualsBrowseService);
  const visualsService: VisualsService = useInjection(VisualsService);

  const tree: IUseTreeState = useTreeState();
  const { reveal, select } = tree;

  const visuals: ReadonlyArray<XrayAsset> = browseService.visuals.value ?? EMPTY_ARRAY;

  const items: Array<IPathTreeItem<XrayAsset>> = useMemo(
    () =>
      parsePathTree(
        visuals.map((asset: XrayAsset) => ({ path: asset.logicalPath, payload: asset })),
        LOGICAL_PATH_SEPARATOR
      ),
    [visuals]
  );

  const source: Nullable<AssetSource> = visualsService.selected?.source ?? null;
  const openItemId: Nullable<string> = source?.kind === EAssetSource.ASSET ? toFileItemId(source.logicalPath) : null;

  const onOpenPath = useCallback(
    (logicalPath: string) => {
      // Selection is written from what was asked for, never derived from what the viewport ended up holding: a
      // model that fails to load leaves its row selected, beside the failure's own retry.
      reveal(toFileItemId(logicalPath));

      void visualsService.openAsset(logicalPath, browseService.rootPaths);
    },
    [browseService.rootPaths, reveal, visualsService]
  );

  // Marks a visual that was read out of an archive, which is all a row says beyond its name.
  const onRenderVisualLabel = useCallback(
    (item: ITreeNode<XrayAsset>) => (
      <TreeRowLabel
        label={item.label}
        caption={item.payload && !isLooseContainer(item.payload.container) ? ARCHIVED_CAPTION : null}
        captionTitle={"Read from an archive volume"}
      />
    ),
    []
  );

  const onOpenAsset = useCallback((asset: XrayAsset) => onOpenPath(asset.logicalPath), [onOpenPath]);

  const onSelectAsset = useCallback((item: ITreeNode<XrayAsset>) => select(item.id), [select]);

  const onActivateAsset = useCallback(
    (item: ITreeNode<XrayAsset>) => {
      if (item.payload) {
        onOpenPath(item.payload.logicalPath);
      }
    },
    [onOpenPath]
  );

  const toVisualSearchText = useCallback((asset: XrayAsset): string => asset.logicalPath, []);

  const toVisualRow = useCallback((asset: XrayAsset): IEditorSearchResultRow => toPathSearchRow(asset.logicalPath), []);

  return (
    <EditorSearchMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Visuals"}
      searchLabel={"Filter visuals"}
      resultsLabel={"Visual search results"}
      items={visuals}
      toSearchText={toVisualSearchText}
      toRow={toVisualRow}
      onSelect={onOpenAsset}
    >
      {items.length ? (
        <VirtualizedTree<XrayAsset>
          ariaLabel={"Visuals"}
          icons={VISUAL_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          activeId={openItemId}
          renderLabel={onRenderVisualLabel}
          onSelect={onSelectAsset}
          onActivate={onActivateAsset}
          onToggleExpanded={tree.toggleExpanded}
        />
      ) : (
        <EmptyListing
          label={browseService.visuals.isLoading ? "Listing visuals…" : "No visuals found under this root."}
        />
      )}
    </EditorSearchMenu>
  );
}
