import { WbSunnyIcon } from "@xrf/ui/icons/wb-sunny";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { WEATHER_EDITOR_HELP } from "./help";

export const WEATHER_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit weather cycles, effects and environment settings",
    group: EToolGroupId.ENVIRONMENT,
    help: WEATHER_EDITOR_HELP,
    icon: <WbSunnyIcon />,
    id: EToolId.WEATHER_EDITOR,
    label: "Weather editor",
    path: "/weather-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
