import { default as MapIcon } from "@mui/icons-material/Map";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { SPAWN_EDITOR_HELP } from "./help";

export const SPAWN_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse and edit a packed spawn file",
    group: EApplicationGroupId.SPAWNS,
    icon: <MapIcon />,
    help: SPAWN_EDITOR_HELP,
    id: EApplicationId.SPAWN_EDITOR,
    label: "Spawn editor",
    path: "/spawn-editor",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
