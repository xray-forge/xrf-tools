import { FolderOpenIcon } from "@xrf/ui/icons/folder-open";
import { InfoIcon } from "@xrf/ui/icons/info";

import { IShellPanel } from "@/core/shell/panels/shell-panel";

import { ArchiveFileDetailsPanel } from "./file-details";
import { ArchivesMenu } from "./tree";

/** The archives explorer's panels, named where anything outside its components can address one. */
export enum EArchivePanelId {
  DETAILS = "details",
  FILES = "archives",
}

/** What the archives explorer contributes to the panel stripe. */
export const ARCHIVE_EXPLORER_PANELS: Array<IShellPanel> = [
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
];
