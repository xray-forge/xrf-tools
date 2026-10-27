import { GrassIcon } from "@xrf/ui/icons/grass";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_DETAILS_COMPILER_HELP } from "./help";

export const LEVEL_DETAILS_COMPILER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build and light grass and detail-object data",
    group: EToolGroupId.LEVEL,
    help: LEVEL_DETAILS_COMPILER_HELP,
    icon: <GrassIcon />,
    id: EToolId.LEVEL_DETAILS_COMPILER,
    label: "Level details compiler",
    path: "/level-details-compiler",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
