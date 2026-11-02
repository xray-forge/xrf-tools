import { useInjection } from "@wirestate/react";
import { XrayAssetContainer } from "@xrf/ipc/types/xrf-vfs";
import { LOGICAL_PATH_SEPARATOR } from "@xrf/path";
import { Nullable, Optional } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyListing } from "@xrf/ui/layout/empty-listing";
import {
  ARCHIVED_CAPTION,
  getDirectoryItemPath,
  ITreeNode,
  IUseTreeState,
  toFileItemId,
  TreeRowLabel,
  useTreeState,
  VirtualizedTree,
} from "@xrf/ui/tree";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { IArchiveEntry, IArchiveTreeItem, parseTree, toArchiveSelectionItemId } from "@/core/archive/lib";
import { isLooseContainer } from "@/core/assets/lib";
import { EditorSearchMenu } from "@/core/shell/chrome/EditorSearchMenu";
import { toPathSearchRow } from "@/core/shell/chrome/EditorSearchMenu/path-search-row";
import { ArchivePreviewService } from "@/tools/archives-explorer/services/archive-preview";
import { ArchivesService } from "@/tools/archives-explorer/services/archives";

import { ARCHIVE_TREE_ICONS, toSearchText } from "./ArchivesMenu.utils";

/**
 * Browses archive files and selects extraction directories through a searchable tree.
 */
export function ArchivesMenu({
  "data-testid": dataTestId = "archives-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);
  const previewService: ArchivePreviewService = useInjection(ArchivePreviewService);

  const isWriting: boolean = archivesService.isWriting;
  const files: Array<IArchiveEntry> = archivesService.entries;
  const openItemId: Nullable<string> = toArchiveSelectionItemId(previewService.selection);

  const tree: IUseTreeState = useTreeState();
  const { reveal, select } = tree;

  const items: Array<IArchiveTreeItem> = useMemo(() => parseTree(files, LOGICAL_PATH_SEPARATOR), [files]);

  const onOpenEntry = useCallback(
    (entry: IArchiveEntry) => {
      if (isWriting) {
        return;
      }

      // Opened from the filter rather than from the tree, so the tree is told where the user landed. Written on
      // the request, not derived from what came back, so a failed read leaves the row selected to retry.
      reveal(toFileItemId(entry.name));

      void previewService.selectFile(entry);
    },
    [isWriting, previewService, reveal]
  );

  // Only a world knows where a file is read from: inside a volume set no entry carries a container, every entry is
  // archived anyway, and a caption on every row would say nothing. A loose file is then the row without the caption,
  // which in a world is the copy a mod installed.
  const onRenderArchiveLabel = useCallback((item: ITreeNode<IArchiveEntry>) => {
    const container: Optional<XrayAssetContainer> = item.payload?.container;

    return (
      <TreeRowLabel
        label={item.label}
        caption={container && !isLooseContainer(container) ? ARCHIVED_CAPTION : null}
        captionTitle={"Read from an archive volume"}
      />
    );
  }, []);

  const onSelectItem = useCallback((item: ITreeNode<IArchiveEntry>) => select(item.id), [select]);

  const onActivateItem = useCallback(
    (item: ITreeNode<IArchiveEntry>) => {
      if (isWriting) {
        return;
      }

      if (item.payload) {
        onOpenEntry(item.payload);

        return;
      }

      const directoryPath: Nullable<string> = getDirectoryItemPath(item.id);

      // The synthetic root node stands for the whole tree, which the backend spells as an empty prefix rather than a
      // literal path - which is what `getDirectoryItemPath` answers for it.
      if (directoryPath !== null) {
        previewService.selectDirectory(directoryPath);
      }
    },
    [isWriting, onOpenEntry, previewService]
  );

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
      title={"Files"}
      searchLabel={"Filter archive files"}
      placeholder={"Filter files"}
      resultsLabel={"Archive search results"}
      items={files}
      toSearchText={toSearchText}
      toRow={(entry) => toPathSearchRow(entry.name)}
      isActivationDisabled={isWriting}
      onSelect={onOpenEntry}
    >
      {items.length ? (
        <VirtualizedTree<IArchiveEntry>
          ariaLabel={"Archive files"}
          icons={ARCHIVE_TREE_ICONS}
          renderLabel={onRenderArchiveLabel}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          activeId={openItemId}
          onSelect={onSelectItem}
          onActivate={onActivateItem}
          onToggleExpanded={tree.toggleExpanded}
        />
      ) : (
        <EmptyListing label={"No archive files found."} />
      )}
    </EditorSearchMenu>
  );
}
