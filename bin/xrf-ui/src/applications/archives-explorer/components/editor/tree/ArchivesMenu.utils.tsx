import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";

import { IArchiveEntry } from "@/core/archive";
import { isLooseContainer } from "@/core/assets/lib";
import { XrayAssetContainer } from "@/core/bindings/types/xrf-vfs";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { ITreeIconDecoration, IVirtualizedTreeIcons } from "@/core/ui/tree/VirtualizedTree";
import { Nullable, Optional } from "@/lib/types/general";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
export const ARCHIVE_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionIcon />,
};

export function toSearchText(entry: IArchiveEntry): string {
  return entry.name;
}

/**
 * How a row is tinted by where the engine would read it from.
 *
 * A colour rather than a second icon set, because the tree reuses one hoisted element per row kind and the icon
 * inherits `currentColor`: tinting costs no new element and no render.
 *
 * @param item - Row the tree is about to draw.
 * @returns How to tint its icon, or null to leave it in the neutral colour.
 */
export function decorateArchiveIcon(item: ITreeNode<IArchiveEntry>): Nullable<ITreeIconDecoration> {
  const container: Optional<XrayAssetContainer> = item.payload?.container;

  if (!container) {
    return null;
  }

  return isLooseContainer(container)
    ? { color: "secondary.main", title: "Loose file" }
    : { color: "primary.main", title: "Archived entry" };
}
