import { useInjection } from "@wirestate/react";
import { TextureSource } from "@xrf/ipc/types/xrf-app-textures";
import { LOGICAL_PATH_SEPARATOR } from "@xrf/path";
import { Nullable, Optional } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FolderIcon, FolderOpenIcon, ImageIcon } from "@xrf/ui/icons";
import { EmptyListing } from "@xrf/ui/layout/EmptyListing";
import { IEditorSearchResultRow } from "@xrf/ui/shell/editor/EditorSearchResults";
import { IPathTreeItem, parsePathTree, toFileItemId } from "@xrf/ui/tree/path-tree";
import { ITreeNode } from "@xrf/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@xrf/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@xrf/ui/tree/VirtualizedTree";
import { ReactElement, ReactNode, useCallback, useMemo, useState } from "react";

import { TextureBadgeFilters } from "@/applications/textures-explorer/components/tree/TextureBadgeFilters";
import {
  countTextureBadges,
  ETextureBadge,
  filterTextureNodes,
  ITextureNode,
  toTextureNodePath,
} from "@/applications/textures-explorer/lib/texture-catalog";
import { TextureCatalogService } from "@/applications/textures-explorer/services/texture-catalog.service";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { toPathSearchRow } from "@/core/shell/editor/EditorSearchMenu/path-search-row";
import { getTextureSourceKey } from "@/core/textures/lib/texture-identity";
import { TextureSelectionService } from "@/core/textures/services/texture-selection";

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
}: BaseComponentProps): ReactElement {
  const catalogService: TextureCatalogService = useInjection(TextureCatalogService);
  const selectionService: TextureSelectionService = useInjection(TextureSelectionService);

  const tree: IUseTreeState = useTreeState();
  const { reveal, select } = tree;

  const [badges, setBadges] = useState<ReadonlySet<ETextureBadge>>(() => new Set());

  const nodes: Array<ITextureNode> = catalogService.nodes;

  // Counted over every node rather than the filtered ones, so a chip keeps saying how much it would show.
  const counts: ReadonlyMap<ETextureBadge, number> = useMemo(() => countTextureBadges(nodes), [nodes]);
  const filtered: Array<ITextureNode> = useMemo(() => filterTextureNodes(nodes, badges), [badges, nodes]);

  const items: Array<IPathTreeItem<ITextureNode>> = useMemo(
    () =>
      parsePathTree(
        filtered.map((node: ITextureNode) => ({ path: toTextureNodePath(node), payload: node })),
        LOGICAL_PATH_SEPARATOR
      ),
    [filtered]
  );

  // Which row the open texture is, matched by the address the listing opened it with rather than by the name the
  // backend labelled it with: a loose file is described by the engine reference its own tree implies for it, which is
  // not what a listing addressed by path calls it.
  const openItemId: Nullable<string> = useMemo(() => {
    const source: Optional<TextureSource> = selectionService.selected.value?.source;

    if (!source) {
      return null;
    }

    const key: string = getTextureSourceKey(source);
    const open: Optional<ITextureNode> = nodes.find((node: ITextureNode) => getTextureSourceKey(node.source) === key);

    return open ? toFileItemId(toTextureNodePath(open)) : null;
  }, [nodes, selectionService.selected.value?.source]);

  const onOpenNode = useCallback(
    (node: ITextureNode) => {
      // Selection is written from what was asked for, never derived from what the panels ended up holding: a texture
      // that fails to describe leaves its row selected, beside the failure's own retry.
      reveal(toFileItemId(toTextureNodePath(node)));

      void catalogService.select(node.source);
    },
    [reveal, catalogService]
  );

  const onSelectNode = useCallback((item: ITreeNode<ITextureNode>) => select(item.id), [select]);

  const onActivateNode = useCallback(
    (item: ITreeNode<ITextureNode>) => {
      if (item.payload) {
        onOpenNode(item.payload);
      }
    },
    [onOpenNode]
  );

  const toTextureRow = useCallback(
    (node: ITextureNode): IEditorSearchResultRow => toPathSearchRow(toTextureNodePath(node)),
    []
  );

  return (
    <EditorSearchMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Textures"}
      searchLabel={"Filter textures"}
      resultsLabel={"Texture search results"}
      items={filtered}
      header={<TextureBadgeFilters counts={counts} selected={badges} onChange={setBadges} />}
      toSearchText={toTextureNodePath}
      toRow={toTextureRow}
      onSelect={onOpenNode}
    >
      {items.length ? (
        <VirtualizedTree<ITextureNode>
          ariaLabel={"Textures"}
          icons={TEXTURE_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          activeId={openItemId}
          renderLabel={renderTextureLabel}
          onSelect={onSelectNode}
          onActivate={onActivateNode}
          onToggleExpanded={tree.toggleExpanded}
        />
      ) : (
        <EmptyListing label={describeEmptyTextureTree(catalogService.catalog.isLoading, nodes.length, badges.size)} />
      )}
    </EditorSearchMenu>
  );
}
