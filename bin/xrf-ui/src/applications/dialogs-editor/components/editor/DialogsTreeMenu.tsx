import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ForumIcon } from "@mui/icons-material/Forum";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { IDialogTreeEntry, IDialogTreeLeaf, toDialogTreeEntries } from "@/applications/dialogs-editor/lib/dialog-tree";
import { DialogsService, IDialogSelection } from "@/applications/dialogs-editor/services/dialogs";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSearchResults, IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EditorSideMenu } from "@/core/shell/editor/EditorSideMenu";
import {
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
}: BaseComponentProps = {}): ReactElement {
  const dialogsService: DialogsService = useInjection(DialogsService);

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const entries: Array<IDialogTreeEntry> = useMemo(
    () => toDialogTreeEntries(dialogsService.project.value),
    [dialogsService.project.value]
  );

  const items: Array<IPathTreeItem<IDialogTreeLeaf>> = useMemo(
    () => parsePathTree(entries, LOGICAL_PATH_SEPARATOR),
    [entries]
  );

  const onOpenLeaf = useCallback(
    (leaf: IDialogTreeLeaf) => void dialogsService.selectDialog(leaf.logicalPath, leaf.id),
    [dialogsService]
  );

  const search: IUseRankedSearch<{ path: string; payload: IDialogTreeLeaf }> = useRankedSearch({
    items: entries,
    // Ranked on the dialog id alone: it is what a writer knows the dialog by, and including the file
    // would let a filename match float dialogs whose own ids do not match at all.
    toSearchText: (it) => it.payload.id,
    onSelect: (it) => onOpenLeaf(it.payload),
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<{ path: string; payload: IDialogTreeLeaf }>) => ({
        id: result.item.path,
        label: result.item.payload.id,
        description: splitLogicalPath(result.item.payload.logicalPath).name,
      })),
    [search.results]
  );

  const selection: Nullable<IDialogSelection> = dialogsService.selection;

  /** The tree path of the selected dialog, which is what both the highlight and the reveal need. */
  const selectedPath: Nullable<string> = useMemo(
    () =>
      entries.find((it) => it.payload.id === selection?.id && it.payload.logicalPath === selection?.logicalPath)
        ?.path ?? null,
    [entries, selection]
  );

  const selectedItemId: Nullable<string> = selectedPath ? toFileItemId(selectedPath) : null;

  /** Result rows are keyed by their tree path, which is what makes one row address one dialog. */
  const onSelectResult = useCallback(
    (rowId: string) => {
      const entry = entries.find((it) => it.path === rowId);

      if (entry) {
        onOpenLeaf(entry.payload);
      }
    },
    [entries, onOpenLeaf]
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
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      header={
        <EditorSearchHeader
          title={"Dialogs"}
          count={entries.length}
          query={search.query}
          placeholder={"Filter dialogs"}
          ariaLabel={"Filter dialogs"}
          onClear={search.clear}
          onKeyDown={search.onInputKeyDown}
          onQueryChange={search.setQuery}
        />
      }
    >
      {search.isSearching ? (
        <EditorSearchResults
          ariaLabel={"Dialog search results"}
          isStale={search.isStale}
          emptyLabel={`No dialogs match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          onHoverIndex={search.setActiveIndex}
          onSelect={onSelectResult}
        />
      ) : (
        <VirtualizedTree
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          ariaLabel={"Dialogs"}
          icons={DIALOG_TREE_ICONS}
          onToggleExpanded={tree.toggleExpanded}
          onSelect={onSelectItem}
          onActivate={onActivateItem}
        />
      )}
    </EditorSideMenu>
  );
}
