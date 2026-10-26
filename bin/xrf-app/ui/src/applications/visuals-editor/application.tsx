import { default as AccessibilityNewIcon } from "@mui/icons-material/AccessibilityNew";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { VISUALS_EDITOR_HELP } from "./help";

export const VISUALS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit model surfaces, skeletons and motion properties",
    group: EApplicationGroupId.VISUALS,
    help: VISUALS_EDITOR_HELP,
    icon: <AccessibilityNewIcon />,
    id: EApplicationId.VISUALS_EDITOR,
    label: "Visuals editor",
    path: "/visuals-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
