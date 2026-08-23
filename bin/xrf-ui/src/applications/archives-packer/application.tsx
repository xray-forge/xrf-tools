import { default as ArchiveIcon } from "@mui/icons-material/Archive";
import { lazy } from "react";

import { PackerService } from "@/applications/archives-packer/services/packer";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const ARCHIVES_PACKER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() =>
    import("./ArchivesPackerApplication").then((it) => ({ default: it.ArchivesPackerApplication }))
  ),
  container: { bindings: [PackerService] },
  preload: () => import("./ArchivesPackerApplication"),
  description: "Pack a directory into game archives",
  group: EApplicationGroupId.ARCHIVES,
  icon: <ArchiveIcon />,
  id: EApplicationId.ARCHIVES_PACKER,
  label: "Archives packer",
  path: "/archives-packer",
  status: EApplicationStatus.READY,
};
