import { default as ArchiveIcon } from "@mui/icons-material/Archive";

import { ARCHIVES_PACKER_HELP } from "@/applications/archives-packer/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const ARCHIVES_PACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Pack a directory into game archives",
    group: EApplicationGroupId.ARCHIVES,
    help: ARCHIVES_PACKER_HELP,
    icon: <ArchiveIcon />,
    id: EApplicationId.ARCHIVES_PACKER,
    label: "Archives packer",
    path: "/archives-packer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
