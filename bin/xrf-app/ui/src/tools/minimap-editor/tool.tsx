import { MapIcon } from "@xrf/ui/icons/map";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { MINIMAP_EDITOR_HELP } from "./help";

export const MINIMAP_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Align a level map image to its level and edit its world bounds",
    group: EToolGroupId.LEVEL,
    help: MINIMAP_EDITOR_HELP,
    icon: <MapIcon />,
    id: EToolId.MINIMAP_EDITOR,
    label: "Minimap editor",
    path: "/minimap-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
