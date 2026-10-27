import { SwapHorizIcon } from "@xrf/ui/icons/swap-horiz";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { EXPORTS_EXPLORER_HELP } from "./help";

export const EXPORTS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse TypeScript extern declarations in an XRF project",
    group: EToolGroupId.SCRIPTS,
    icon: <SwapHorizIcon />,
    help: EXPORTS_EXPLORER_HELP,
    id: EToolId.EXPORTS_EXPLORER,
    label: "TypeScript exports explorer",
    path: "/exports-explorer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
