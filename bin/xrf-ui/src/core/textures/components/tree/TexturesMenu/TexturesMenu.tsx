import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ImageIcon } from "@mui/icons-material/Image";
import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useMemo, useState } from "react";

import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { TextureBadgeFilters } from "@/core/textures/components/tree/TextureBadgeFilters";
import {
  countTextureBadges,
  ETextureBadge,
  filterTextureNodes,
  ITextureNode,
} from "@/core/textures/lib/texture-catalog";
import { TextureCatalogService } from "@/core/textures/services/catalog";
import { IPathTreeItem, parsePathTree, splitLogicalPath, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { StyledComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

import { describeEmptyTextureTree } from "./TexturesMenu.utils";
import { TextureTreeLabel } from "./TextureTreeLabel";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
const TEXTURE_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <ImageIcon />,
};

/**
 * Hoisted so a row is not handed a new renderer on every keystroke in the filter.
 *
 * @param item - Node being labelled.
 * @returns The row's label.
 */
function renderTextureLabel(item: ITreeNode<ITextureNode>): ReactNode {
  return <TextureTreeLabel item={item} />;
}

/**
 * Every texture of the browsed roots, as a tree, with the declared bump pairs folded under the textures declaring
 * them.
 */
export function TexturesMenu({
  "data-testid": dataTestId = "textures-menu",
  id,
  className,
  sx,
}: StyledComponentProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const [badges, setBadges] = useState<ReadonlySet<ETextureBadge>>(() => new Set());

  const nodes: Array<ITextureNode> = catalogService.nodes;

  // Counted over every node rather than the filtered ones, so a chip keeps saying how much it would show.
  const counts: ReadonlyMap<ETextureBadge, number> = useMemo(() => countTextureBadges(nodes), [nodes]);
  const filtered: Array<ITextureNode> = useMemo(() => filterTextureNodes(nodes, badges), [badges, nodes]);

  const items: Array<IPathTreeItem<ITextureNode>> = useMemo(
    () =>
      parsePathTree(
        filtered.map((node: ITextureNode) => ({ path: node.reference, payload: node })),
        LOGICAL_PATH_SEPARATOR
      ),
    [filtered]
  );

  const onOpenNode = useCallback(
    (node: ITextureNode) => {
      // Selection is written from what was asked for, never derived from what the panels ended up holding: a texture
      // that fails to describe leaves its row selected, beside the failure's own retry.
      reveal(toFileItemId(node.reference));

      void catalogService.select(node.source);
    },
    [reveal, catalogService]
  );

  const onSelectNode = useCallback((item: ITreeNode<ITextureNode>) => tree.select(item.id), [tree]);

  const onActivateNode = useCallback(
    (item: ITreeNode<ITextureNode>) => {
      if (item.payload) {
        onOpenNode(item.payload);
      }
    },
    [onOpenNode]
  );

  return (
    <EditorSearchMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      title={"Textures"}
      searchLabel={"Filter textures"}
      resultsLabel={"Texture search results"}
      items={filtered}
      toSearchText={(node: ITextureNode) => node.reference}
      toRow={(node) => {
        const { name, directory } = splitLogicalPath(node.reference);

        return { id: node.reference, label: name, description: directory ?? undefined };
      }}
      onSelect={onOpenNode}
      header={<TextureBadgeFilters counts={counts} selected={badges} onChange={setBadges} />}
    >
      {items.length ? (
        <VirtualizedTree<ITextureNode>
          ariaLabel={"Textures"}
          icons={TEXTURE_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          renderLabel={renderTextureLabel}
          onSelect={onSelectNode}
          onActivate={onActivateNode}
          onToggleExpanded={tree.toggleExpanded}
        />
      ) : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            {describeEmptyTextureTree(catalogService.catalog.isLoading, nodes.length, badges.size)}
          </Typography>
        </Box>
      )}
    </EditorSearchMenu>
  );
}
