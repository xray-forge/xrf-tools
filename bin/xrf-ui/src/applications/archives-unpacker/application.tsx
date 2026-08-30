import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";
import { lazy } from "react";

import { ARCHIVES_UNPACKER_HELP } from "@/applications/archives-unpacker/help";
import { UnpackerService } from "@/applications/archives-unpacker/services/unpacker";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const ARCHIVES_UNPACKER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./ArchivesUnpackerApplication").then((it) => ({ default: it.ArchivesUnpackerApplication }))
  ),
  container: { bindings: [UnpackerService] },
  preload: () => import("./ArchivesUnpackerApplication"),
  description: "Extract game archives into a directory",
  group: EApplicationGroupId.ARCHIVES,
  help: ARCHIVES_UNPACKER_HELP,
  icon: <UnarchiveIcon />,
  id: EApplicationId.ARCHIVES_UNPACKER,
  label: "Archives unpacker",
  path: "/archives-unpacker",
  status: EApplicationStatus.READY,
};
