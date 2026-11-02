import { MapIcon } from "@xrf/ui/icons/map";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SPAWN_EDITOR_HELP } from "./help";

export const SPAWN_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse and edit a packed spawn file",
    group: EToolGroupId.SPAWNS,
    icon: <MapIcon />,
    help: SPAWN_EDITOR_HELP,
    id: EToolId.SPAWN_EDITOR,
    label: "Spawn editor",
    path: "/spawn-editor",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
