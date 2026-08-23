import { default as AssignmentIcon } from "@mui/icons-material/Assignment";
import { lazy } from "react";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";

export const TASKS_EXPLORER_APPLICATION: IApplicationDescriptor = {
  Component: lazy(() => import("./TasksExplorerApplication").then((it) => ({ default: it.TasksExplorerApplication }))),
  preload: () => import("./TasksExplorerApplication"),
  description: "Browse quest tasks and the condlists driving them",
  group: EApplicationGroupId.GAMEPLAY,
  icon: <AssignmentIcon />,
  id: EApplicationId.TASKS_EXPLORER,
  label: "Tasks explorer",
  path: "/tasks-explorer",
  status: EApplicationStatus.PLANNED,
};
