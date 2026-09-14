import { Box, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useMemo } from "react";

import {
  ARCHIVE_TREE_ICONS,
  toSearchText,
} from "@/applications/archives-explorer/components/editor/tree/ArchivesMenu.utils";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IArchiveEntry, IArchiveTreeItem, parseTree, toArchiveSelectionItemId } from "@/core/archive/lib";
import { isLooseContainer } from "@/core/assets/lib";
import { XrayAssetContainer } from "@/core/ipc/types/xrf-vfs";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import { getDirectoryItemPath, splitLogicalPath, toFileItemId } from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ARCHIVED_CAPTION, TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * Browses archive files and selects extraction directories through a searchable tree.
 */
export function ArchivesMenu({
  "data-testid": dataTestId = "archives-menu",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const archivesService: ArchivesService = useInjection(ArchivesService);

  const isWriting: boolean = archivesService.isWriting;
  const files: Array<IArchiveEntry> = archivesService.entries;
  const openItemId: Nullable<string> = toArchiveSelectionItemId(archivesService.selection);

  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const items: Array<IArchiveTreeItem> = useMemo(() => parseTree(files, LOGICAL_PATH_SEPARATOR), [files]);

  const onOpenEntry = useCallback(
    (entry: IArchiveEntry) => {
      if (isWriting) {
        return;
      }

      // Opened from the filter rather than from the tree, so the tree is told where the user landed. Written on
      // the request, not derived from what came back, so a failed read leaves the row selected to retry.
      reveal(toFileItemId(entry.name));

      void archivesService.selectArchiveFile(entry);
    },
    [archivesService, isWriting, reveal]
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

  const onSelectItem = useCallback((item: ITreeNode<IArchiveEntry>) => tree.select(item.id), [tree]);

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
        archivesService.selectArchiveDirectory(directoryPath);
      }
    },
    [archivesService, isWriting, onOpenEntry]
  );

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
      toRow={(entry) => {
        const { name, directory } = splitLogicalPath(entry.name);

        return { id: entry.name, label: name, description: directory ?? undefined };
      }}
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
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            No archive files found.
          </Typography>
        </Box>
      )}
    </EditorSearchMenu>
  );
}
