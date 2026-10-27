import { DescriptionIcon, FolderIcon, FolderOpenIcon } from "@xrf/ui/icons";
import { IVirtualizedTreeIcons } from "@xrf/ui/tree/VirtualizedTree";

import { IArchiveEntry } from "@/core/archive/lib";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
export const ARCHIVE_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionIcon />,
};

export function toSearchText(entry: IArchiveEntry): string {
  return entry.name;
}
