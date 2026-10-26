import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SPAWN_COMPILER_HELP } from "./help";

export const SPAWN_COMPILER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build game spawn data from level sources and graphs",
    group: EApplicationGroupId.SPAWNS,
    help: SPAWN_COMPILER_HELP,
    icon: <AccountTreeIcon />,
    id: EApplicationId.SPAWN_COMPILER,
    label: "Spawn compiler",
    path: "/spawn-compiler",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
