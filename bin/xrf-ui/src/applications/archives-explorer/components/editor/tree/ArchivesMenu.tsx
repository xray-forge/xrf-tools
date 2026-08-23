import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { Box, Typography } from "@mui/material";
import { RichTreeView } from "@mui/x-tree-view";
import { useInjection } from "@wirestate/react";
import { ReactElement, SyntheticEvent, useCallback, useMemo, useState } from "react";

import { ArchiveTreeItem } from "@/applications/archives-explorer/components/editor/tree/ArchiveTreeItem";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IArchiveTreeItem, parseTree, TArchiveSelection } from "@/core/archive";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { ISearchResult, IUseRankedSearch, useRankedSearch } from "@/core/search/lib";
import { EditorSearchHeader } from "@/core/shell/editor/EditorSearchHeader";
import { EditorSearchResults, IEditorSearchResultRow } from "@/core/shell/editor/EditorSearchResults";
import { EditorSideMenu } from "@/core/shell/editor/EditorSideMenu";
import {
  getDirectoryItemPath,
  getFileItemPath,
  LOGICAL_PATH_SEPARATOR,
  toDirectoryItemId,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable, Optional } from "@/lib/types/general";

export interface IArchivesMenuProps extends BaseComponentProps {}

export function ArchivesMenu({
  "data-testid": dataTestId = "archives-menu",
  id,
  className,
}: IArchivesMenuProps = {}): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const [expandedItems, setExpandedItems] = useState<Array<string>>([]);

  const files: Array<ArchiveFileDescriptor> = archivesService.files;

  const items: Array<IArchiveTreeItem> = useMemo(() => parseTree(files, LOGICAL_PATH_SEPARATOR), [files]);

  // Selecting again while a read or a write is in flight starts work that the previous one will
  // outlive, and the tree would show a selection whose content is still the old one.
  const isBusy: boolean = archivesService.isBusy;

  const onSelectDescriptor = useCallback(
    (descriptor: ArchiveFileDescriptor) => {
      void archivesService.selectArchiveFile(descriptor);
    },
    [archivesService]
  );

  const search: IUseRankedSearch<ArchiveFileDescriptor> = useRankedSearch({
    items: files,
    toSearchText: (it) => it.name,
    onSelect: onSelectDescriptor,
  });

  const rows: Array<IEditorSearchResultRow> = useMemo(
    () =>
      search.results.map((result: ISearchResult<ArchiveFileDescriptor>) => {
        const separatorAt: number = result.item.name.lastIndexOf("\\");

        return {
          id: result.item.name,
          label: separatorAt === -1 ? result.item.name : result.item.name.slice(separatorAt + 1),
          description: separatorAt === -1 ? undefined : result.item.name.slice(0, separatorAt),
        };
      }),
    [search.results]
  );

  const selection: TArchiveSelection = archivesService.selection;
  const selectedItem: Nullable<string> =
    selection.kind === "file"
      ? toFileItemId(selection.descriptor.name)
      : selection.kind === "directory"
        ? toDirectoryItemId(selection.path)
        : null;

  const onSelectPath = useCallback(
    (path: string) => {
      if (isBusy) {
        return;
      }

      const descriptor: Optional<ArchiveFileDescriptor> = archivesService.project.value?.files[path];

      if (descriptor) {
        onSelectDescriptor(descriptor);
      }
    },
    [archivesService, isBusy, onSelectDescriptor]
  );

  const onSelectItem = useCallback(
    (_: Nullable<SyntheticEvent>, itemId: Nullable<string>) => {
      if (isBusy) {
        return;
      }

      const filePath: Nullable<string> = getFileItemPath(itemId);

      if (filePath) {
        onSelectPath(filePath);

        return;
      }

      const directoryPath: Nullable<string> = getDirectoryItemPath(itemId);

      // The synthetic root node stands for the whole archive, which the backend spells as an empty prefix rather than a
      // literal path - which is what `getDirectoryItemPath` answers for it.
      if (directoryPath !== null) {
        archivesService.selectArchiveDirectory(directoryPath);
      }
    },
    [archivesService, isBusy, onSelectPath]
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
          isDisabled={isBusy}
          isStale={search.isStale}
          emptyLabel={`No files match ${search.query.trim()}.`}
          rows={rows}
          total={search.total}
          activeIndex={search.activeIndex}
          onHoverIndex={search.setActiveIndex}
          onSelect={onSelectPath}
        />
      ) : items.length ? (
        <Box sx={{ padding: 0.5 }}>
          <RichTreeView
            isItemSelectionDisabled={() => isBusy}
            items={items}
            expandedItems={expandedItems}
            selectedItems={selectedItem}
            expansionTrigger={"content"}
            slots={{
              item: ArchiveTreeItem,
              collapseIcon: FolderOpenIcon,
              expandIcon: FolderIcon,
              endIcon: DescriptionIcon,
            }}
            onExpandedItemsChange={(_, next: Array<string>) => setExpandedItems(next)}
            onSelectedItemsChange={onSelectItem}
          />
        </Box>
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
