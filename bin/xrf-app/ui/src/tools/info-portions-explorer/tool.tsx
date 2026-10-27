import { HubIcon } from "@xrf/ui/icons/hub";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

export const INFO_PORTIONS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse info portions and what gives or requires them",
    group: EToolGroupId.GAMEPLAY,
    icon: <HubIcon />,
    id: EToolId.INFO_PORTIONS_EXPLORER,
    label: "Info portions explorer",
    path: "/info-portions-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
