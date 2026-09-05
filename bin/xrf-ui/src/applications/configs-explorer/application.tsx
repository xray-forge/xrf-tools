import { default as FolderOpenIcon } from "@mui/icons-material/FolderOpen";

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
    icon: <FolderOpenIcon />,
    id: EApplicationId.CONFIGS_EXPLORER,
    label: "Configs explorer",
    path: "/configs-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
