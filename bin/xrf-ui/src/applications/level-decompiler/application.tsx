import { default as AccountTreeIcon } from "@mui/icons-material/AccountTree";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { LEVEL_DECOMPILER_HELP } from "./help";

export const LEVEL_DECOMPILER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Reconstruct editable scenes from compiled game levels",
    group: EApplicationGroupId.LEVEL,
    help: LEVEL_DECOMPILER_HELP,
    icon: <AccountTreeIcon />,
    id: EApplicationId.LEVEL_DECOMPILER,
    label: "Level decompiler",
    path: "/level-decompiler",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
