import { Inventory2Icon } from "@xrf/ui/icons/inventory2";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SPAWN_PACKER_HELP } from "./help";

export const SPAWN_PACKER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build a spawn file from unpacked chunks",
    group: EToolGroupId.SPAWNS,
    icon: <Inventory2Icon />,
    help: SPAWN_PACKER_HELP,
    id: EToolId.SPAWN_PACKER,
    label: "Spawn packer",
    path: "/spawn-packer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
