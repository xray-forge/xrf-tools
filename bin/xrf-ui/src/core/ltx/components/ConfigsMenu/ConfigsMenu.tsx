import { Box, Typography } from "@mui/material";
import { ReactElement, useCallback, useEffect, useMemo } from "react";

import { LtxInventoryFile } from "@/core/ipc/types/xrf-ltx-inspect";
import { CONFIG_TREE_ICONS, decorateConfigIcon } from "@/core/ltx/components/ConfigsMenu/ConfigsMenu.utils";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import {
  getFileItemPath,
  IPathTreeItem,
  parsePathTree,
  splitLogicalPath,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ARCHIVED_CAPTION, TreeRowLabel } from "@/core/ui/tree/TreeRowLabel";
import { IUseTreeState, useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { StyledComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

interface IConfigsMenuProps extends StyledComponentProps {
  files: ReadonlyArray<LtxInventoryFile>;
  /** Config that is on screen, whoever opened it. */
  selected: Nullable<string>;
  onOpen: (path: string) => void;
}

/**
 * The project as a tree of configs, with what each one is to it.
 */
export function ConfigsMenu({
  "data-testid": dataTestId = "configs-menu",
  id,
  className,
  sx,
  files,
  selected,
  onOpen,
}: IConfigsMenuProps): ReactElement {
  const tree: IUseTreeState = useTreeState();
  const { reveal } = tree;

  const items: Array<IPathTreeItem<LtxInventoryFile>> = useMemo(
    () =>
      parsePathTree(
        files.map((file: LtxInventoryFile) => ({ path: file.path, payload: file })),
        LOGICAL_PATH_SEPARATOR
      ),
    [files]
  );

  const searchable: Array<LtxInventoryFile> = useMemo(() => [...files], [files]);

  // Whether the engine reads it out of an archive. What the config is to the project - entry point, scheme, patch -
  // is the icon's tint instead, because it is the dimension that varies.
  const onRenderConfigLabel = useCallback(
    (item: ITreeNode<LtxInventoryFile>) => (
      <TreeRowLabel
        label={item.label}
        caption={item.payload && !item.payload.isPhysical ? ARCHIVED_CAPTION : null}
        captionTitle={"Read from an archive volume; nothing can write to it in place"}
      />
    ),
    []
  );

  const onSelectNode = useCallback((item: ITreeNode<LtxInventoryFile>) => tree.select(item.id), [tree]);

  const onActivateNode = useCallback(
    (item: ITreeNode<LtxInventoryFile>) => {
      const path: Nullable<string> = getFileItemPath(item.id);

      if (path) {
        onOpen(path);
      }
    },
    [onOpen]
  );

  const onOpenFile = useCallback((file: LtxInventoryFile) => onOpen(file.path), [onOpen]);

  useEffect(() => {
    if (selected) {
      reveal(toFileItemId(selected));
    }
  }, [reveal, selected]);

  return (
    <EditorSearchMenu<LtxInventoryFile>
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={sx}
      title={"Configs"}
      searchLabel={"Filter configs"}
      resultsLabel={"Config search results"}
      items={searchable}
      toSearchText={(file: LtxInventoryFile) => file.path}
      toRow={(file: LtxInventoryFile) => {
        const { name, directory } = splitLogicalPath(file.path);

        return { id: file.path, label: name, description: directory ?? undefined };
      }}
      onSelect={onOpenFile}
    >
      {items.length ? (
        <VirtualizedTree<LtxInventoryFile>
          ariaLabel={"Configs"}
          icons={CONFIG_TREE_ICONS}
          decorateIcon={decorateConfigIcon}
          items={items}
          expandedIds={tree.expandedIds}
          selectedId={tree.selectedId}
          renderLabel={onRenderConfigLabel}
          onToggleExpanded={tree.toggleExpanded}
          onSelect={onSelectNode}
          onActivate={onActivateNode}
        />
      ) : (
        <Box sx={{ padding: 2, textAlign: "center" }}>
          <Typography variant={"body2"} sx={{ color: "text.secondary" }}>
            This project holds no configs.
          </Typography>
        </Box>
      )}
    </EditorSearchMenu>
  );
}
