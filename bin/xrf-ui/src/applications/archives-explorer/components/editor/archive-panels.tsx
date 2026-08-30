import { default as InfoIcon } from "@mui/icons-material/Info";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";

import { ArchiveCollisionsPanel } from "@/applications/archives-explorer/components/editor/collisions/ArchiveCollisionsPanel";
import { ArchiveFileDetailsPanel } from "@/applications/archives-explorer/components/editor/file-details/ArchiveFileDetailsPanel";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { IEditorPanel } from "@/core/shell/panel/context";

export function createArchiveEditorPanels(archivesService: ArchivesService): Array<IEditorPanel> {
  return [
    {
      id: "details",
      label: "File details",
      icon: <InfoIcon />,
      isOpenByDefault: false,
      render: () => <ArchiveFileDetailsPanel archivesService={archivesService} />,
    },
    {
      id: "collisions",
      label: "Unreachable files",
      icon: <WarningIcon />,
      isOpenByDefault: false,
      render: () => <ArchiveCollisionsPanel archivesService={archivesService} />,
    },
  ];
}
