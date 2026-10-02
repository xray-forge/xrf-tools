import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { LtxInventoryFile } from "@/core/ipc/types/xrf-ltx-inspect";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { toPathSearchRow } from "@/core/shell/editor/EditorSearchMenu/path-search-row";
import { IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EmptyListing } from "@/core/ui/layout";
import { IPathTreeItem, parsePathTree, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ARCHIVED_CAPTION, TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";

import { CONFIG_TREE_ICONS, decorateConfigIcon } from "./ConfigsMenu.utils";

interface IConfigsMenuProps extends BaseComponentProps {
  files: ReadonlyArray<LtxInventoryFile>;
  /** Config that is on screen, whoever opened it. */
  selected: Nullable<string>;
  onOpen: (path: string) => void;
}

/**
 * The project as a tree of configs, with what each one is to it.
 */
export function ConfigsMenu({
  "data-testid": dataTestId = "configs-menu",
  id,
  className,
  files,
  selected,
  onOpen,
}: IConfigsMenuProps): ReactElement {
  const tree: IUseTreeState = useTreeState();
  const openItemId: Nullable<string> = selected ? toFileItemId(selected) : null;
  const { reveal, select } = tree;

  const items: Array<IPathTreeItem<LtxInventoryFile>> = useMemo(
    () =>
      parsePathTree(
        files.map((file: LtxInventoryFile) => ({ path: file.path, payload: file })),
        LOGICAL_PATH_SEPARATOR
      ),
    [files]
  );

  // Whether the engine reads it out of an archive. What the config is to the project - entry point, scheme, patch -
  // is the icon's tint instead, because it is the dimension that varies.
  const onRenderConfigLabel = useCallback(
    (item: ITreeNode<LtxInventoryFile>) => (
      <TreeRowLabel
        label={item.label}
        caption={item.payload && !item.payload.isPhysical ? ARCHIVED_CAPTION : null}
        captionTitle={"Read from an archive volume; nothing can write to it in place"}
      />
    ),
    []
  );

  const onSelectNode = useCallback((item: ITreeNode<LtxInventoryFile>) => select(item.id), [select]);

  const onActivateNode = useCallback(
    (item: ITreeNode<LtxInventoryFile>) => {
      if (item.payload) {
        onOpen(item.payload.path);
      }
    },
    [onOpen]
  );

  const onOpenFile = useCallback((file: LtxInventoryFile) => onOpen(file.path), [onOpen]);

  const toConfigSearchText = useCallback((file: LtxInventoryFile): string => file.path, []);

  const toConfigRow = useCallback((file: LtxInventoryFile): IEditorSearchResultRow => toPathSearchRow(file.path), []);

  useEffect(() => {
    if (openItemId) {
      reveal(openItemId);
    }
  }, [openItemId, reveal]);

  return (
    <EditorSearchMenu<LtxInventoryFile>
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Configs"}
      searchLabel={"Filter configs"}
      resultsLabel={"Config search results"}
      items={files}
      toSearchText={toConfigSearchText}
      toRow={toConfigRow}
      onSelect={onOpenFile}
    >
      {items.length ? (
        <VirtualizedTree<LtxInventoryFile>
          ariaLabel={"Configs"}
          icons={CONFIG_TREE_ICONS}
          decorateIcon={decorateConfigIcon}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          activeId={openItemId}
          renderLabel={onRenderConfigLabel}
          onToggleExpanded={tree.toggleExpanded}
          onSelect={onSelectNode}
          onActivate={onActivateNode}
        />
      ) : (
        <EmptyListing label={"This project holds no configs."} />
      )}
    </EditorSearchMenu>
  );
}
