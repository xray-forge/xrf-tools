import { LightbulbIcon } from "@xrf/ui/icons/lightbulb";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LIGHT_ANIMATIONS_EDITOR_HELP } from "./help";

export const LIGHT_ANIMATIONS_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Edit animated light colors and timing",
    group: EToolGroupId.ENVIRONMENT,
    help: LIGHT_ANIMATIONS_EDITOR_HELP,
    icon: <LightbulbIcon />,
    id: EToolId.LIGHT_ANIMATIONS_EDITOR,
    label: "Light animations editor",
    path: "/light-animations-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
