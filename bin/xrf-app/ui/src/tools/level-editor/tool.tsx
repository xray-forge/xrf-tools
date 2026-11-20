import { EditLocationAltIcon } from "@xrf/ui/icons/edit-location-alt";

import { FRAME_LEVEL_SELECTION_KEYBIND_COMMAND } from "@/core/level/commands";
import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_EDITOR_HELP } from "./help";

export const LEVEL_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "View an SDK level scene and its objects",
    group: EToolGroupId.LEVEL,
    help: LEVEL_EDITOR_HELP,
    icon: <EditLocationAltIcon />,
    id: EToolId.LEVEL_EDITOR,
    keybindCommands: [FRAME_LEVEL_SELECTION_KEYBIND_COMMAND],
    label: "Level editor",
    path: "/level-editor",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
