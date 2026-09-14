import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ForumIcon } from "@mui/icons-material/Forum";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { IDialogTreeEntry, IDialogTreeLeaf } from "@/applications/dialogs-editor/lib/dialog-tree";
import { DialogsService } from "@/applications/dialogs-editor/services/dialogs";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { IPathTreeItem, parsePathTree, splitLogicalPath, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

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

  const entries: Array<IDialogTreeEntry> = dialogsService.entries;
  const selectedItemId: Nullable<string> = dialogsService.selectedPath
    ? toFileItemId(dialogsService.selectedPath)
    : null;

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const items: Array<IPathTreeItem<IDialogTreeLeaf>> = useMemo(
    () => parsePathTree(entries, LOGICAL_PATH_SEPARATOR),
    [entries]
  );

  const onOpenLeaf = useCallback(
    (leaf: IDialogTreeLeaf) => void dialogsService.selectDialog(leaf.logicalPath, leaf.id),
    [dialogsService]
  );

  const onSelectItem = useCallback((item: ITreeNode<IDialogTreeLeaf>) => tree.select(item.id), [tree]);

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
      toSearchText={(entry) => entry.payload.id}
      toRow={(entry) => ({
        id: entry.path,
        label: entry.payload.id,
        description: splitLogicalPath(entry.payload.logicalPath).name,
      })}
      onSelect={(entry) => onOpenLeaf(entry.payload)}
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
