import { default as BlurOnIcon } from "@mui/icons-material/BlurOn";

import { PARTICLES_EXPLORER_HELP } from "@/applications/particles-explorer/help";
import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const PARTICLES_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse particle effects, groups and dependencies",
    group: EApplicationGroupId.PARTICLES,
    help: PARTICLES_EXPLORER_HELP,
    icon: <BlurOnIcon />,
    id: EApplicationId.PARTICLES_EXPLORER,
    label: "Particles explorer",
    path: "/particles-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
