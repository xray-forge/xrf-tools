import { default as LightbulbIcon } from "@mui/icons-material/Lightbulb";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { LIGHT_ANIMATIONS_EDITOR_HELP } from "./help";

export const LIGHT_ANIMATIONS_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit animated light colors and timing",
    group: EApplicationGroupId.ENVIRONMENT,
    help: LIGHT_ANIMATIONS_EDITOR_HELP,
    icon: <LightbulbIcon />,
    id: EApplicationId.LIGHT_ANIMATIONS_EDITOR,
    label: "Light animations editor",
    path: "/light-animations-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
