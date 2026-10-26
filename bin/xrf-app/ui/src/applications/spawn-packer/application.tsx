import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SPAWN_PACKER_HELP } from "./help";

export const SPAWN_PACKER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build a spawn file from unpacked chunks",
    group: EApplicationGroupId.SPAWNS,
    icon: <Inventory2Icon />,
    help: SPAWN_PACKER_HELP,
    id: EApplicationId.SPAWN_PACKER,
    label: "Spawn packer",
    path: "/spawn-packer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
