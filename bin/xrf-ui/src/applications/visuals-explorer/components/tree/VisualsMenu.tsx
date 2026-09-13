import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";
import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { VisualsBrowseService } from "@/applications/visuals-explorer/services/browse";
import { VisualsService } from "@/applications/visuals-explorer/services/visuals";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import {
  getFileItemPath,
  IPathTreeItem,
  parsePathTree,
  splitLogicalPath,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ARCHIVED_CAPTION, TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { StyledComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

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
  sx,
}: StyledComponentProps): ReactElement {
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
        caption={item.payload?.container.kind === "archive" ? ARCHIVED_CAPTION : null}
        captionTitle={"Read from an archive volume"}
      />
    ),
    []
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
    <EditorSearchMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      title={"Visuals"}
      searchLabel={"Filter visuals"}
      resultsLabel={"Visual search results"}
      items={visuals}
      toSearchText={(asset: XrayAsset) => asset.logicalPath}
      toRow={(asset) => {
        const { name, directory } = splitLogicalPath(asset.logicalPath);

        return { id: asset.logicalPath, label: name, description: directory ?? undefined };
      }}
      onSelect={(asset) => onOpenPath(asset.logicalPath)}
    >
      {items.length ? (
        <VirtualizedTree<XrayAsset>
          ariaLabel={"Visuals"}
          icons={VISUAL_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          renderLabel={onRenderVisualLabel}
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
    </EditorSearchMenu>
  );
}
