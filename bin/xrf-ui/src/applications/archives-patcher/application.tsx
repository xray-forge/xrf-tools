import { default as DifferenceIcon } from "@mui/icons-material/Difference";

import { ARCHIVES_PATCHER_HELP } from "@/applications/archives-patcher/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const ARCHIVES_PATCHER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Pack what your gamedata changes about a game as an overriding patch",
    group: EApplicationGroupId.ARCHIVES,
    help: ARCHIVES_PATCHER_HELP,
    icon: <DifferenceIcon />,
    id: EApplicationId.ARCHIVES_PATCHER,
    label: "Archives patcher",
    path: "/archives-patcher",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
