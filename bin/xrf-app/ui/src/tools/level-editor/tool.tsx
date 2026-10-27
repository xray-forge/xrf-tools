import { EditLocationAltIcon } from "@xrf/ui/icons/edit-location-alt";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_EDITOR_HELP } from "./help";

export const LEVEL_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit level geometry and scene objects",
    group: EToolGroupId.LEVEL,
    help: LEVEL_EDITOR_HELP,
    icon: <EditLocationAltIcon />,
    id: EToolId.LEVEL_EDITOR,
    label: "Level editor",
    path: "/level-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
