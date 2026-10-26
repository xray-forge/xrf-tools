import { default as AssignmentIcon } from "@mui/icons-material/Assignment";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const TASKS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse quest tasks and the condlists driving them",
    group: EApplicationGroupId.GAMEPLAY,
    icon: <AssignmentIcon />,
    id: EApplicationId.TASKS_EXPLORER,
    label: "Tasks explorer",
    path: "/tasks-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
