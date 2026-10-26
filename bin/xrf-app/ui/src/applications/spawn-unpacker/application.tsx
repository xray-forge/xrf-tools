import { default as UnarchiveIcon } from "@mui/icons-material/Unarchive";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SPAWN_UNPACKER_HELP } from "./help";

export const SPAWN_UNPACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Extract a spawn file into editable chunks",
    group: EApplicationGroupId.SPAWNS,
    icon: <UnarchiveIcon />,
    help: SPAWN_UNPACKER_HELP,
    id: EApplicationId.SPAWN_UNPACKER,
    label: "Spawn unpacker",
    path: "/spawn-unpacker",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
