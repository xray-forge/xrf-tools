import { LayersIcon } from "@xrf/ui/icons/layers";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { GAME_MATERIALS_EDITOR_HELP } from "./help";

export const GAME_MATERIALS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit physical materials and their interaction pairs",
    group: EToolGroupId.MATERIALS,
    help: GAME_MATERIALS_EDITOR_HELP,
    icon: <LayersIcon />,
    id: EToolId.GAME_MATERIALS_EDITOR,
    label: "Game materials editor",
    path: "/game-materials-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
