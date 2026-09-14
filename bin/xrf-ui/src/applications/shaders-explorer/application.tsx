import { default as GradientIcon } from "@mui/icons-material/Gradient";

import { SHADERS_EXPLORER_HELP } from "@/applications/shaders-explorer/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const SHADERS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Inspect render shaders and level-compiler shader definitions",
    group: EApplicationGroupId.SHADERS,
    help: SHADERS_EXPLORER_HELP,
    icon: <GradientIcon />,
    id: EApplicationId.SHADERS_EXPLORER,
    label: "Shaders explorer",
    path: "/shaders-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
