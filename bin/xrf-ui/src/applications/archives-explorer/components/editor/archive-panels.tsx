import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";
import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";

import { IEditorPanel } from "@/core/shell/editor-shell";

import { ArchiveCollisionsPanel } from "./collisions";
import { ArchiveFileDetailsPanel } from "./file-details";
import { ArchivesMenu } from "./tree";

/** The archives explorer's panels, named where anything outside its components can address one. */
export enum EArchivePanelId {
  COLLISIONS = "collisions",
  DETAILS = "details",
  FILES = "archives",
}

/** What the archives explorer contributes to the panel stripe. */
export const ARCHIVE_EXPLORER_PANELS: Array<IEditorPanel> = [
  {
    id: EArchivePanelId.FILES,
    label: "Archives",
    icon: <FolderOpenIcon />,
    side: "left",
    isOpenByDefault: true,
    render: () => <ArchivesMenu />,
  },
  {
    id: EArchivePanelId.DETAILS,
    label: "File details",
    icon: <InfoIcon />,
    isOpenByDefault: false,
    render: () => <ArchiveFileDetailsPanel />,
  },
  {
    id: EArchivePanelId.COLLISIONS,
    label: "Unreachable files",
    icon: <WarningIcon />,
    isOpenByDefault: false,
    render: () => <ArchiveCollisionsPanel />,
  },
];
