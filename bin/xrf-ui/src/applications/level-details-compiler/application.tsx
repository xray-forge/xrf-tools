import { default as GrassIcon } from "@mui/icons-material/Grass";

import { LEVEL_DETAILS_COMPILER_HELP } from "@/applications/level-details-compiler/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const LEVEL_DETAILS_COMPILER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Build and light grass and detail-object data",
    group: EApplicationGroupId.LEVEL,
    help: LEVEL_DETAILS_COMPILER_HELP,
    icon: <GrassIcon />,
    id: EApplicationId.LEVEL_DETAILS_COMPILER,
    label: "Level details compiler",
    path: "/level-details-compiler",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
