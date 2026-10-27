import { GradientIcon } from "@xrf/ui/icons/gradient";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { SHADERS_EXPLORER_HELP } from "./help";

export const SHADERS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Inspect render shaders and level-compiler shader definitions",
    group: EToolGroupId.SHADERS,
    help: SHADERS_EXPLORER_HELP,
    icon: <GradientIcon />,
    id: EToolId.SHADERS_EXPLORER,
    label: "Shaders explorer",
    path: "/shaders-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
