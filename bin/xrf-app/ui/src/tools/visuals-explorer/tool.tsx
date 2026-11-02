import { ViewInArIcon } from "@xrf/ui/icons/view-in-ar";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { VISUALS_EXPLORER_HELP } from "./help";

export const VISUALS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse and preview game visuals in 3D",
    group: EToolGroupId.VISUALS,
    help: VISUALS_EXPLORER_HELP,
    icon: <ViewInArIcon />,
    id: EToolId.VISUALS_EXPLORER,
    label: "Visuals explorer",
    path: "/visuals-explorer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
