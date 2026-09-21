import { default as WbSunnyIcon } from "@mui/icons-material/WbSunny";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { WEATHER_EDITOR_HELP } from "./help";

export const WEATHER_EDITOR_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Edit weather cycles, effects and environment settings",
    group: EApplicationGroupId.ENVIRONMENT,
    help: WEATHER_EDITOR_HELP,
    icon: <WbSunnyIcon />,
    id: EApplicationId.WEATHER_EDITOR,
    label: "Weather editor",
    path: "/weather-editor",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
