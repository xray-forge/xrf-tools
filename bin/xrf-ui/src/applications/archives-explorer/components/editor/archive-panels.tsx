import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";

import { IEditorPanel } from "@/core/shell/editor-shell";

import { ArchiveCollisionsPanel } from "./collisions";
import { ArchiveFileDetailsPanel } from "./file-details";

/** What the archives editor contributes to the panel stripe. */
export const ARCHIVE_EDITOR_PANELS: Array<IEditorPanel> = [
  {
    id: "details",
    label: "File details",
    icon: <InfoIcon />,
    isOpenByDefault: false,
    render: () => <ArchiveFileDetailsPanel />,
  },
  {
    id: "collisions",
    label: "Unreachable files",
    icon: <WarningIcon />,
    isOpenByDefault: false,
    render: () => <ArchiveCollisionsPanel />,
  },
];
