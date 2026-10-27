import { SwapHorizIcon } from "@xrf/ui/icons/swap-horiz";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { VISUALS_CONVERTER_HELP } from "./help";

export const VISUALS_CONVERTER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Convert models and animations between game and editable formats",
    group: EToolGroupId.VISUALS,
    help: VISUALS_CONVERTER_HELP,
    icon: <SwapHorizIcon />,
    id: EToolId.VISUALS_CONVERTER,
    label: "Visuals converter",
    path: "/visuals-converter",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
