import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SPAWN_COMPILER_HELP } from "./help";

export const SPAWN_COMPILER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build game spawn data from level sources and graphs",
    group: EToolGroupId.SPAWNS,
    help: SPAWN_COMPILER_HELP,
    icon: <AccountTreeIcon />,
    id: EToolId.SPAWN_COMPILER,
    label: "Spawn compiler",
    path: "/spawn-compiler",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
