import { UnarchiveIcon } from "@xrf/ui/icons/unarchive";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SPAWN_UNPACKER_HELP } from "./help";

export const SPAWN_UNPACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Extract a spawn file into editable chunks",
    group: EToolGroupId.SPAWNS,
    icon: <UnarchiveIcon />,
    help: SPAWN_UNPACKER_HELP,
    id: EToolId.SPAWN_UNPACKER,
    label: "Spawn unpacker",
    path: "/spawn-unpacker",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
