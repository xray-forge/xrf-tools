import { useInjection } from "@wirestate/react";
import { LOGICAL_PATH_SEPARATOR, splitLogicalPath } from "@xrf/path";
import { Nullable } from "@xrf/types";
import { IEditorSearchResultRow } from "@xrf/ui/editor/search-results";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DescriptionIcon } from "@xrf/ui/icons/description";
import { FolderOpenIcon } from "@xrf/ui/icons/folder-open";
import { ForumIcon } from "@xrf/ui/icons/forum";
import {
  IPathTreeItem,
  ITreeNode,
  IUseTreeState,
  IVirtualizedTreeIcons,
  parsePathTree,
  toFileItemId,
  useTreeState,
  VirtualizedTree,
} from "@xrf/ui/tree";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { IDialogTreeEntry, IDialogTreeLeaf } from "@/tools/dialogs-editor/lib/dialog-tree";
import { DialogViewService } from "@/tools/dialogs-editor/services/dialog-view";
import { DialogsService } from "@/tools/dialogs-editor/services/dialogs";

/**
 * Hoisted so the tree is handed the same icons every render rather than a fresh set.
 */
const DIALOG_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <DescriptionIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <ForumIcon />,
};

/**
 * Every dialog in the project, grouped under the file declaring it.
 */
export function DialogsTreeMenu({
  "data-testid": dataTestId = "dialogs-tree-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);
  const viewService: DialogViewService = useInjection(DialogViewService);

  const entries: Array<IDialogTreeEntry> = dialogsService.entries;
  const selectedItemId: Nullable<string> = viewService.selectedPath ? toFileItemId(viewService.selectedPath) : null;

  const tree: IUseTreeState = useTreeState();
  const { reveal, select } = tree;

  const items: Array<IPathTreeItem<IDialogTreeLeaf>> = useMemo(
    () => parsePathTree(entries, LOGICAL_PATH_SEPARATOR),
    [entries]
  );

  const onOpenLeaf = useCallback(
    (leaf: IDialogTreeLeaf) => void viewService.selectDialog(leaf.logicalPath, leaf.id),
    [viewService]
  );

  const onOpenEntry = useCallback((entry: IDialogTreeEntry) => onOpenLeaf(entry.payload), [onOpenLeaf]);

  const onSelectItem = useCallback((item: ITreeNode<IDialogTreeLeaf>) => select(item.id), [select]);

  const onActivateItem = useCallback(
    (item: ITreeNode<IDialogTreeLeaf>) => {
      // Leaves only, and a leaf is what carries a dialog. `VirtualizedTree` has already expanded a file node by
      // the time this runs, and a dialog file has nothing else to open - the canvas draws one dialog, not a
      // whole file.
      if (item.payload) {
        onOpenLeaf(item.payload);
      }
    },
    [onOpenLeaf]
  );

  const toDialogSearchText = useCallback((entry: IDialogTreeEntry): string => entry.payload.id, []);

  const toDialogRow = useCallback(
    (entry: IDialogTreeEntry): IEditorSearchResultRow => ({
      id: entry.path,
      label: entry.payload.id,
      description: splitLogicalPath(entry.payload.logicalPath).name,
    }),
    []
  );

  useEffect(() => {
    if (selectedItemId) {
      reveal(selectedItemId);
    }
  }, [reveal, selectedItemId]);

  return (
    <EditorSearchMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Dialogs"}
      searchLabel={"Filter dialogs"}
      resultsLabel={"Dialog search results"}
      items={entries}
      toSearchText={toDialogSearchText}
      toRow={toDialogRow}
      onSelect={onOpenEntry}
    >
      <VirtualizedTree
        items={items}
        expandedIds={tree.expandedIds}
        selectedId={tree.selectedId}
        activeId={selectedItemId}
        ariaLabel={"Dialogs"}
        icons={DIALOG_TREE_ICONS}
        onToggleExpanded={tree.toggleExpanded}
        onSelect={onSelectItem}
        onActivate={onActivateItem}
      />
    </EditorSearchMenu>
  );
}
