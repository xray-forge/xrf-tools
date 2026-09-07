import { default as InfoIcon } from "@mui/icons-material/Info";

import { SpawnRowDetailsPanel } from "@/applications/spawn-editor/components/editor/details/SpawnRowDetailsPanel";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { SpawnFileService } from "@/core/spawn/services";

export function createSpawnEditorPanels(spawnFileService: SpawnFileService): Array<IEditorPanel> {
  return [
    {
      id: "details",
      label: "Row details",
      icon: <InfoIcon />,
      isOpenByDefault: false,
      render: () => <SpawnRowDetailsPanel spawnFileService={spawnFileService} />,
    },
  ];
}
