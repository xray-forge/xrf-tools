import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { lazy } from "react";

import { ARCHIVES_EXPLORER_HELP } from "@/applications/archives-explorer/help";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { AssetService } from "@/core/assets/services";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { VisualLoadService } from "@/core/visuals/services";

export const ARCHIVES_EXPLORER_APPLICATION: IApplicationDescriptor = {
  container: { bindings: [AssetService, ArchivesService, VisualLoadService] },
  Component: lazy(() =>
    import("./ArchivesExplorerApplication").then((it) => ({ default: it.ArchivesExplorerApplication }))
  ),
  preload: () => import("./ArchivesExplorerApplication"),
  description: "Browse files stored in game archives",
  group: EApplicationGroupId.ARCHIVES,
  help: ARCHIVES_EXPLORER_HELP,
  icon: <ArchiveIcon />,
  id: EApplicationId.ARCHIVES_EXPLORER,
  label: "Archives explorer",
  path: "/archives-explorer",
  status: EApplicationStatus.READY,
};
