import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";

import { LtxInventoryFile } from "@/core/ipc/types/xrf-ltx-inspect";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ITreeIconDecoration, IVirtualizedTreeIcons } from "@/core/ui/tree/VirtualizedTree";
import { Nullable } from "@/lib/types/general";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
export const CONFIG_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionIcon />,
};

/**
 * How a row is tinted by what the config is to the project.
 *
 * @param item - Row the tree is about to draw.
 * @returns How to tint its icon, or null to leave it in the neutral colour.
 */
export function decorateConfigIcon(item: ITreeNode<LtxInventoryFile>): Nullable<ITreeIconDecoration> {
  // Switched on the role's own discriminator rather than tested against it three times, so adding a role to the
  // inventory is a compiler error here instead of a row that silently loses its mark.
  switch (item.payload?.role.kind) {
    case "entryPoint":
      return { color: "primary.main", title: "Entry point: nothing includes it, so it resolves on its own" };

    case "schemeFile":
      return { color: "secondary.main", title: "Scheme declaration: what a section bound to it may hold" };

    case "attachment":
      return { color: "success.main", title: "Patch file: it changes another config rather than standing on its own" };

    case "included":
      return null;

    // A directory row, which stands for no config at all.
    case undefined:
      return null;
  }
}
