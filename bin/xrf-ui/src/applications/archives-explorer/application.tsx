import { default as ArchiveIcon } from "@mui/icons-material/Archive";

import { ARCHIVES_EXPLORER_HELP } from "@/applications/archives-explorer/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const ARCHIVES_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse files stored in game archives",
    group: EApplicationGroupId.ARCHIVES,
    help: ARCHIVES_EXPLORER_HELP,
    icon: <ArchiveIcon />,
    id: EApplicationId.ARCHIVES_EXPLORER,
    label: "Archives explorer",
    path: "/archives-explorer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
