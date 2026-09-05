import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ImageIcon } from "@mui/icons-material/Image";
import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useCallback, useMemo, useState } from "react";

import { TextureBadgeFilters } from "@/applications/textures-explorer/components/editor/tree/TextureBadgeFilters";
import {
  countTextureBadges,
  ETextureBadge,
  filterTextureNodes,
  ITextureNode,
} from "@/applications/textures-explorer/lib/texture-catalog";
import { TexturesService } from "@/applications/textures-explorer/services/textures";
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
import { StyledComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
  const texturesService: TexturesService = useInjection(TexturesService);

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const [badges, setBadges] = useState<ReadonlySet<ETextureBadge>>(() => new Set());

  const nodes: Array<ITextureNode> = texturesService.nodes;

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

  const onOpenReference = useCallback(
    (reference: string) => {
      // Selection is written from what was asked for, never derived from what the panels ended up holding: a texture
      // that fails to describe leaves its row selected, beside the failure's own retry.
      reveal(toFileItemId(reference));

      void texturesService.select(reference);
    },
    [reveal, texturesService]
  );

  const search: IUseRankedSearch<ITextureNode> = useRankedSearch({
    items: filtered,
    toSearchText: (node: ITextureNode) => node.reference,
    onSelect: (node: ITextureNode) => onOpenReference(node.reference),
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<ITextureNode>) => {
        const { name, directory } = splitLogicalPath(result.item.reference);

        return { id: result.item.reference, label: name, description: directory ?? undefined };
      }),
    [search.results]
  );

  const onSelectNode = useCallback((item: ITreeNode<ITextureNode>) => tree.select(item.id), [tree]);

  const onActivateNode = useCallback(
    (item: ITreeNode<ITextureNode>) => {
      const reference: Nullable<string> = getFileItemPath(item.id);

      if (reference) {
        onOpenReference(reference);
      }
    },
    [onOpenReference]
  );

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      header={
        <>
          <EditorSearchHeader
            title={"Textures"}
            count={filtered.length}
            query={search.query}
            placeholder={"Filter textures"}
            ariaLabel={"Filter textures"}
            onClear={search.clear}
            onKeyDown={search.onInputKeyDown}
            onQueryChange={search.setQuery}
          />

          <TextureBadgeFilters counts={counts} selected={badges} onChange={setBadges} />
        </>
      }
    >
      {search.isSearching ? (
        <EditorSearchResults
          ariaLabel={"Texture search results"}
          isStale={search.isStale}
          emptyLabel={`No textures match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          onHoverIndex={search.setActiveIndex}
          onSelect={onOpenReference}
        />
      ) : items.length ? (
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
            {describeEmptyTextureTree(texturesService.catalog.isLoading, nodes.length, badges.size)}
          </Typography>
        </Box>
      )}
    </EditorSideMenu>
  );
}
