import { default as DifferenceIcon } from "@mui/icons-material/Difference";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { ARCHIVES_PATCHER_HELP } from "./help";

export const ARCHIVES_PATCHER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Pack gamedata changes as a patch",
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
