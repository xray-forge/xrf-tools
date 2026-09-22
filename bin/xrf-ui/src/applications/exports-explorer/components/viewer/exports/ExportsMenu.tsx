import { default as DataObjectIcon } from "@mui/icons-material/DataObject";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { ExportDescriptor } from "@/core/ipc/types/xrf-export";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EmptyListing } from "@/core/ui/layout";
import { IPathTreeItem, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { exportGroupsToTree, getExportSearchText, IExportGroup } from "./exports-groups";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
const EXPORT_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DataObjectIcon />,
};

export interface IExportsMenuProps extends BaseComponentProps {
  declarations: Array<ExportDescriptor>;
  groups: ReadonlyArray<IExportGroup>;
  selectedName: Nullable<string>;
  onSelect: (name: string) => void;
}

export function ExportsMenu({
  "data-testid": dataTestId,
  id,
  className,
  declarations,
  groups,
  selectedName,
  onSelect,
}: IExportsMenuProps): ReactElement {
  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const items: Array<IPathTreeItem<ExportDescriptor>> = useMemo(() => exportGroupsToTree(groups), [groups]);

  const onSelectDeclaration = useCallback(
    (declaration: ExportDescriptor) => {
      onSelect(declaration.name);
    },
    [onSelect]
  );

  const openItemId: Nullable<string> = selectedName ? toFileItemId(selectedName) : null;

  const onSelectItem = useCallback((item: ITreeNode<ExportDescriptor>) => tree.select(item.id), [tree]);

  const onActivateItem = useCallback(
    (item: ITreeNode<ExportDescriptor>) => {
      if (item.payload) {
        onSelectDeclaration(item.payload);
      }
    },
    [onSelectDeclaration]
  );

  const toDeclarationSearchText = useCallback((declaration: ExportDescriptor): string => declaration.name, []);

  const toDeclarationRow = useCallback((declaration: ExportDescriptor): IEditorSearchResultRow => {
    const separatorAt: number = declaration.name.lastIndexOf(".");

    return {
      id: declaration.name,
      label: separatorAt === -1 ? declaration.name : declaration.name.slice(separatorAt + 1),
      description: separatorAt === -1 ? undefined : declaration.name.slice(0, separatorAt),
    };
  }, []);

  // The viewer is showing a declaration the tree did not necessarily choose - a filter result, or the first one
  // after a refresh - so the row follows what is on screen rather than being derived from it.
  useEffect(() => {
    if (openItemId) {
      reveal(openItemId);
    }
  }, [openItemId, reveal]);

  return (
    <EditorSearchMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      title={"Exports"}
      searchLabel={"Filter exports"}
      resultsLabel={"Export search results"}
      items={declarations}
      toSearchText={toDeclarationSearchText}
      toRow={toDeclarationRow}
      onSelect={onSelectDeclaration}
      toSecondaryText={getExportSearchText}
    >
      {items.length ? (
        <VirtualizedTree<ExportDescriptor>
          ariaLabel={"Exports"}
          icons={EXPORT_TREE_ICONS}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          activeId={openItemId}
          onSelect={onSelectItem}
          onActivate={onActivateItem}
          onToggleExpanded={tree.toggleExpanded}
        />
      ) : (
        <EmptyListing label={"No externs found."} />
      )}
    </EditorSearchMenu>
  );
}
