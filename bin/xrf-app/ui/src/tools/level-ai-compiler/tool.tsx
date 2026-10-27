import { RouteIcon } from "@xrf/ui/icons/route";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LEVEL_AI_COMPILER_HELP } from "./help";

export const LEVEL_AI_COMPILER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Build AI maps, level graphs and cross tables",
    group: EToolGroupId.LEVEL,
    help: LEVEL_AI_COMPILER_HELP,
    icon: <RouteIcon />,
    id: EToolId.LEVEL_AI_COMPILER,
    label: "Level AI compiler",
    path: "/level-ai-compiler",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
