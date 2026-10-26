import { default as AutoAwesomeIcon } from "@mui/icons-material/AutoAwesome";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { PARTICLES_EDITOR_HELP } from "./help";

export const PARTICLES_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Author particle effects and groups with a live preview",
    group: EApplicationGroupId.PARTICLES,
    help: PARTICLES_EDITOR_HELP,
    icon: <AutoAwesomeIcon />,
    id: EApplicationId.PARTICLES_EDITOR,
    label: "Particles editor",
    path: "/particles-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
