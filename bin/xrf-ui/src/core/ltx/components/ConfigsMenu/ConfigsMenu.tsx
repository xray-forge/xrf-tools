import { Box, Typography } from "@mui/material";
import { ReactElement, useCallback, useMemo } from "react";

import { LtxInventoryFile } from "@/core/bindings/types/xrf-ltx-inspect";
import { ConfigsTreeLabel } from "@/core/ltx/components/ConfigsMenu/ConfigsTreeLabel";
import { EditorSearchMenu } from "@/core/shell/editor/EditorSearchMenu";
import {
  getFileItemPath,
  IPathTreeItem,
  parsePathTree,
  splitLogicalPath,
  toFileItemId,
} from "@/core/ui/tree/path-tree";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { useTreeState } from "@/core/ui/tree/use-tree-state";
import { VirtualizedTree } from "@/core/ui/tree/VirtualizedTree";
import { StyledComponentProps } from "@/lib/dom/element-types";
import { LOGICAL_PATH_SEPARATOR } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

interface IConfigsMenuProps extends StyledComponentProps {
  files: ReadonlyArray<LtxInventoryFile>;
  selected: Nullable<string>;
  onSelect: (path: string) => void;
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
  onSelect,
}: IConfigsMenuProps): ReactElement {
  const { expandedIds, toggleExpanded } = useTreeState();

  const items: Array<IPathTreeItem<LtxInventoryFile>> = useMemo(
    () =>
      parsePathTree(
        files.map((file: LtxInventoryFile) => ({ path: file.path, payload: file })),
        LOGICAL_PATH_SEPARATOR
      ),
    [files]
  );

  const searchable: Array<LtxInventoryFile> = useMemo(() => [...files], [files]);

  const onSelectNode = useCallback(
    (item: ITreeNode<LtxInventoryFile>) => {
      const path: Nullable<string> = getFileItemPath(item.id);

      if (path) {
        onSelect(path);
      }
    },
    [onSelect]
  );

  const onSelectFile = useCallback((file: LtxInventoryFile) => onSelect(file.path), [onSelect]);

  // The whole label, because `VirtualizedTree` renders this in place of the row's text rather than beside it.
  const renderLabel = useCallback((item: ITreeNode<LtxInventoryFile>) => <ConfigsTreeLabel item={item} />, []);

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
      onSelect={onSelectFile}
    >
      {items.length ? (
        <VirtualizedTree<LtxInventoryFile>
          ariaLabel={"Configs"}
          items={items}
          expandedIds={expandedIds}
          selectedId={selected ? toFileItemId(selected) : null}
          renderLabel={renderLabel}
          onToggleExpanded={toggleExpanded}
          onSelect={onSelectNode}
          onActivate={onSelectNode}
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
