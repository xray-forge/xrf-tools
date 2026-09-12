import { default as DescriptionIcon } from "@mui/icons-material/Description";
import { default as FolderIcon } from "@mui/icons-material/Folder";
import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";

import { IArchiveEntry } from "@/core/archive";
import { IVirtualizedTreeIcons } from "@/core/ui/tree/VirtualizedTree";

/** Hoisted so the tree is handed the same icons every render rather than a fresh set. */
export const ARCHIVE_TREE_ICONS: IVirtualizedTreeIcons = {
  collapsed: <FolderIcon />,
  expanded: <FolderOpenIcon />,
  leaf: <DescriptionIcon />,
};

export function toSearchText(entry: IArchiveEntry): string {
  return entry.name;
}
