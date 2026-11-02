import { BlurOnIcon } from "@xrf/ui/icons/blur-on";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { PARTICLES_EXPLORER_HELP } from "./help";

export const PARTICLES_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse particle effects, groups and dependencies",
    group: EToolGroupId.PARTICLES,
    help: PARTICLES_EXPLORER_HELP,
    icon: <BlurOnIcon />,
    id: EToolId.PARTICLES_EXPLORER,
    label: "Particles explorer",
    path: "/particles-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
