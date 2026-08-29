import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IArchiveTreeItem, parseTree } from "@/core/archive";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSearchResults, IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EditorSideMenu } from "@/core/shell/editor/EditorSideMenu";
import {
  getDirectoryItemPath,
  getFileItemPath,
  LOGICAL_PATH_SEPARATOR,
  splitLogicalPath,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { IVirtualizedTreeIcons, VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable, Optional } from "@/lib/types/general";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
const ARCHIVE_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionIcon />,
};

export function ArchivesMenu({
  "data-testid": dataTestId = "archives-menu",
  id,
  className,
}: BaseComponentProps = {}): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const files: Array<ArchiveFileDescriptor> = archivesService.files;

  const items: Array<IArchiveTreeItem> = useMemo(() => parseTree(files, LOGICAL_PATH_SEPARATOR), [files]);

  // Only a write holds an open back: an extraction runs outside the archive and cannot be abandoned, while a read
  // is simply superseded by the next open. Selecting is inert and never waits for anything.
  const isWriting: boolean = archivesService.isWriting;

  const onOpenDescriptor = useCallback(
    (descriptor: ArchiveFileDescriptor) => {
      if (isWriting) {
        return;
      }

      // Opened from the filter rather than from the tree, so the tree is told where the user landed. Written on
      // the request, not derived from what came back, so a failed read leaves the row selected to retry.
      reveal(toFileItemId(descriptor.name));

      void archivesService.selectArchiveFile(descriptor);
    },
    [archivesService, isWriting, reveal]
  );

  const search: IUseRankedSearch<ArchiveFileDescriptor> = useRankedSearch({
    items: files,
    toSearchText: (it) => it.name,
    onSelect: onOpenDescriptor,
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<ArchiveFileDescriptor>) => {
        const { name, directory } = splitLogicalPath(result.item.name);

        return { id: result.item.name, label: name, description: directory ?? undefined };
      }),
    [search.results]
  );

  const onOpenPath = useCallback(
    (path: string) => {
      const descriptor: Optional<ArchiveFileDescriptor> = archivesService.project.value?.files[path];

      if (descriptor) {
        onOpenDescriptor(descriptor);
      }
    },
    [archivesService, onOpenDescriptor]
  );

  const onSelectItem = useCallback((item: ITreeNode<ArchiveFileDescriptor>) => tree.select(item.id), [tree]);

  const onActivateItem = useCallback(
    (item: ITreeNode<ArchiveFileDescriptor>) => {
      if (isWriting) {
        return;
      }

      const itemId: string = item.id;
      const filePath: Nullable<string> = getFileItemPath(itemId);

      if (filePath) {
        onOpenPath(filePath);

        return;
      }

      const directoryPath: Nullable<string> = getDirectoryItemPath(itemId);

      // The synthetic root node stands for the whole archive, which the backend spells as an empty prefix rather than a
      // literal path - which is what `getDirectoryItemPath` answers for it.
      if (directoryPath !== null) {
        archivesService.selectArchiveDirectory(directoryPath);
      }
    },
    [archivesService, isWriting, onOpenPath]
  );

  return (
    <EditorSideMenu
      data-testid={dataTestId}
      id={id}
      className={className}
      header={
        <EditorSearchHeader
          title={"Files"}
          count={files.length}
          query={search.query}
          placeholder={"Filter files"}
          ariaLabel={"Filter archive files"}
          onClear={search.clear}
          onKeyDown={search.onInputKeyDown}
          onQueryChange={search.setQuery}
        />
      }
    >
      {search.isSearching ? (
        <EditorSearchResults
          ariaLabel={"Archive search results"}
          isDisabled={isWriting}
          isStale={search.isStale}
          emptyLabel={`No files match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          onHoverIndex={search.setActiveIndex}
          onSelect={onOpenPath}
        />
      ) : items.length ? (
        <VirtualizedTree<ArchiveFileDescriptor>
          ariaLabel={"Archive files"}
          icons={ARCHIVE_TREE_ICONS}
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
            No archive files found.
          </Typography>
        </Box>
      )}
    </EditorSideMenu>
  );
}
