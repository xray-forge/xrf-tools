import { default as RouteIcon } from "@mui/icons-material/Route";

import { LEVEL_AI_COMPILER_HELP } from "@/applications/level-ai-compiler/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const LEVEL_AI_COMPILER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build AI maps, level graphs and cross tables",
    group: EApplicationGroupId.LEVEL,
    help: LEVEL_AI_COMPILER_HELP,
    icon: <RouteIcon />,
    id: EApplicationId.LEVEL_AI_COMPILER,
    label: "Level AI compiler",
    path: "/level-ai-compiler",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
