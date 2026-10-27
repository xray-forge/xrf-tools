import { AutoAwesomeIcon } from "@xrf/ui/icons/auto-awesome";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { PARTICLES_EDITOR_HELP } from "./help";

export const PARTICLES_EDITOR_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Author particle effects and groups with a live preview",
    group: EToolGroupId.PARTICLES,
    help: PARTICLES_EDITOR_HELP,
    icon: <AutoAwesomeIcon />,
    id: EToolId.PARTICLES_EDITOR,
    label: "Particles editor",
    path: "/particles-editor",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
