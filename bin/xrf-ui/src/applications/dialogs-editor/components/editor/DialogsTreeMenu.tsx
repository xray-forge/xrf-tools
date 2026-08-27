import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as ForumIcon } from "@mui/icons-material/Forum";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useEffect, useMemo, useState } from "react";

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
  toDirectoryItemId,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
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

  const [expandedIds, setExpandedIds] = useState<ReadonlySet<string>>(() => new Set());

  const entries: Array<IDialogTreeEntry> = useMemo(
    () => toDialogTreeEntries(dialogsService.project.value),
    [dialogsService.project.value]
  );

  const items: Array<IPathTreeItem<IDialogTreeLeaf>> = useMemo(
    () => parsePathTree(entries, LOGICAL_PATH_SEPARATOR),
    [entries]
  );

  const onSelectLeaf = useCallback(
    (leaf: IDialogTreeLeaf) => void dialogsService.selectDialog(leaf.logicalPath, leaf.id),
    [dialogsService]
  );

  const search: IUseRankedSearch<{ path: string; payload: IDialogTreeLeaf }> = useRankedSearch({
    items: entries,
    // Ranked on the dialog id alone: it is what a writer knows the dialog by, and including the file
    // would let a filename match float dialogs whose own ids do not match at all.
    toSearchText: (it) => it.payload.id,
    onSelect: (it) => onSelectLeaf(it.payload),
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

  // Reveal whatever is selected. A dialog picked out of the filter results, or restored with the
  // project, otherwise stays hidden inside a collapsed file and the tree shows no selection at all.
  // Additive, so collapsing a file by hand afterwards stays collapsed.
  useEffect(() => {
    const { directory } = selectedPath ? splitLogicalPath(selectedPath) : { directory: null };

    if (directory) {
      setExpandedIds((current: ReadonlySet<string>) => {
        const id: string = toDirectoryItemId(directory);

        return current.has(id) ? current : new Set(current).add(id);
      });
    }
  }, [selectedPath]);

  /** Result rows are keyed by their tree path, which is what makes one row address one dialog. */
  const onSelectResult = useCallback(
    (rowId: string) => {
      const entry = entries.find((it) => it.path === rowId);

      if (entry) {
        onSelectLeaf(entry.payload);
      }
    },
    [entries, onSelectLeaf]
  );

  const onToggleExpanded = useCallback((itemId: string) => {
    setExpandedIds((current: ReadonlySet<string>) => {
      const next: Set<string> = new Set(current);

      if (!next.delete(itemId)) {
        next.add(itemId);
      }

      return next;
    });
  }, []);

  const onSelectItem = useCallback(
    (item: IPathTreeItem<IDialogTreeLeaf>) => {
      // Leaves only. `VirtualizedTree` already expands a directory before reporting it, so toggling
      // here as well would close the file the same click opened. A dialog file has nothing else to
      // open — the canvas draws one dialog, not a whole file — so reporting it does nothing.
      if (item.kind === "file") {
        onSelectLeaf(item.payload);
      }
    },
    [onSelectLeaf]
  );

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
          expandedIds={expandedIds}
          selectedId={selectedItemId}
          ariaLabel={"Dialogs"}
          icons={DIALOG_TREE_ICONS}
          onToggleExpanded={onToggleExpanded}
          onSelect={onSelectItem}
        />
      )}
    </EditorSideMenu>
  );
}
