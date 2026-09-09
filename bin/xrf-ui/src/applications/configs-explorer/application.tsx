import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";

import { CONFIGS_EXPLORER_HELP } from "@/applications/configs-explorer/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const CONFIGS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse LTX configuration files",
    group: EApplicationGroupId.CONFIGS,
    help: CONFIGS_EXPLORER_HELP,
    icon: <FolderOpenIcon />,
    id: EApplicationId.CONFIGS_EXPLORER,
    label: "Configs explorer",
    path: "/configs-explorer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
