import { default as FilterIcon } from "@mui/icons-material/Filter";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { POSTPROCESS_EDITOR_HELP } from "./help";

export const POSTPROCESS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Author postprocess effect curves and previews",
    group: EApplicationGroupId.ENVIRONMENT,
    help: POSTPROCESS_EDITOR_HELP,
    icon: <FilterIcon />,
    id: EApplicationId.POSTPROCESS_EDITOR,
    label: "Postprocess editor",
    path: "/postprocess-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
