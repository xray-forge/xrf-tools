import { AssignmentIcon } from "@xrf/ui/icons/assignment";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const TASKS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse quest tasks and the condlists driving them",
    group: EToolGroupId.GAMEPLAY,
    icon: <AssignmentIcon />,
    id: EToolId.TASKS_EXPLORER,
    label: "Tasks explorer",
    path: "/tasks-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
