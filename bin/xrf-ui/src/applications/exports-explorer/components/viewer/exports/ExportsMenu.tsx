import { default as DataObjectIcon } from "@mui/icons-material/DataObject";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Box, Typography } from "@mui/material";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import {
  exportGroupsToTree,
  getExportSearchText,
  groupExports,
  IExportGroup,
} from "@/applications/exports-explorer/components/viewer/exports/exports-groups";
import { ExportDescriptor } from "@/core/bindings/types/xrf-export";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSearchResults, IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EditorSideMenu } from "@/core/shell/editor/EditorSideMenu";
import { getFileItemPath, IPathTreeItem, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
const EXPORT_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DataObjectIcon />,
};

export interface IExportsMenuProps extends BaseComponentProps {
  declarations: Array<ExportDescriptor>;
  selectedName: Nullable<string>;
  onSelect: (name: string) => void;
}

export function ExportsMenu({ declarations, selectedName, onSelect }: IExportsMenuProps): ReactElement {
  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const groups: Array<IExportGroup> = useMemo(() => groupExports(declarations), [declarations]);
  const items: Array<IPathTreeItem<ExportDescriptor>> = useMemo(() => exportGroupsToTree(groups), [groups]);

  const onSelectDeclaration = useCallback(
    (declaration: ExportDescriptor) => {
      onSelect(declaration.name);
    },
    [onSelect]
  );

  const search: IUseRankedSearch<ExportDescriptor> = useRankedSearch({
    items: declarations,
    toSearchText: (it) => it.name,
    toSecondaryText: getExportSearchText,
    onSelect: onSelectDeclaration,
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<ExportDescriptor>) => {
        const separatorAt: number = result.item.name.lastIndexOf(".");

        return {
          id: result.item.name,
          label: separatorAt === -1 ? result.item.name : result.item.name.slice(separatorAt + 1),
          description: separatorAt === -1 ? undefined : result.item.name.slice(0, separatorAt),
        };
      }),
    [search.results]
  );

  const openItemId: Nullable<string> = selectedName ? toFileItemId(selectedName) : null;

  const onSelectItem = useCallback((item: ITreeNode<ExportDescriptor>) => tree.select(item.id), [tree]);

  const onActivateItem = useCallback(
    (item: ITreeNode<ExportDescriptor>) => {
      // Namespaces activate too, and answer null here, which is what used to be spelled as disabling
      // their selection.
      const name: Nullable<string> = getFileItemPath(item.id);

      if (name) {
        onSelect(name);
      }
    },
    [onSelect]
  );

  // The viewer is showing a declaration the tree did not necessarily choose - a filter result, or the first one
  // after a refresh - so the row follows what is on screen rather than being derived from it.
  useEffect(() => {
    if (openItemId) {
      reveal(openItemId);
    }
  }, [openItemId, reveal]);

  return (
    <EditorSideMenu
      header={
        <EditorSearchHeader
          title={"Exports"}
          count={declarations.length}
          query={search.query}
          placeholder={"Filter exports"}
          ariaLabel={"Filter exports"}
          onClear={search.clear}
          onKeyDown={search.onInputKeyDown}
          onQueryChange={search.setQuery}
        />
      }
    >
      {search.isSearching ? (
        <EditorSearchResults
          ariaLabel={"Export search results"}
          emptyLabel={`No exports match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          isStale={search.isStale}
          onHoverIndex={search.setActiveIndex}
          onSelect={onSelect}
        />
      ) : items.length ? (
        <VirtualizedTree<ExportDescriptor>
          ariaLabel={"Exports"}
          icons={EXPORT_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          onSelect={onSelectItem}
          onActivate={onActivateItem}
          onToggleExpanded={tree.toggleExpanded}
        />
      ) : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            No externs found.
          </Typography>
        </Box>
      )}
    </EditorSideMenu>
  );
}
