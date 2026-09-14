import { default as EditLocationAltIcon } from "@mui/icons-material/EditLocationAlt";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

export const LEVEL_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit level geometry and scene objects",
    group: EApplicationGroupId.LEVEL,
    icon: <EditLocationAltIcon />,
    id: EApplicationId.LEVEL_EDITOR,
    label: "Level editor",
    path: "/level-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
