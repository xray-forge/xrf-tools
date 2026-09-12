import { default as ViewInArIcon } from "@mui/icons-material/ViewInAr";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { VISUALS_EXPLORER_HELP } from "./help";

export const VISUALS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse and preview game visuals in 3D",
    group: EApplicationGroupId.VISUALS,
    help: VISUALS_EXPLORER_HELP,
    icon: <ViewInArIcon />,
    id: EApplicationId.VISUALS_EXPLORER,
    label: "Visuals explorer",
    path: "/visuals-explorer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
