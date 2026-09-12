import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";

import { ArchiveCollisionsPanel } from "@/applications/archives-explorer/components/editor/collisions/ArchiveCollisionsPanel";
import { ArchiveFileDetailsPanel } from "@/applications/archives-explorer/components/editor/file-details/ArchiveFileDetailsPanel";
import { IEditorPanel } from "@/core/shell/editor-shell";

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
