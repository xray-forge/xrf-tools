import { FormatAlignLeftIcon } from "@xrf/ui/icons/format-align-left";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { CONFIGS_FORMATTER_HELP } from "./help";

export const CONFIGS_FORMATTER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Check or format LTX configuration files",
    group: EToolGroupId.CONFIGS,
    help: CONFIGS_FORMATTER_HELP,
    icon: <FormatAlignLeftIcon />,
    id: EToolId.CONFIGS_FORMATTER,
    label: "Configs formatter",
    path: "/configs-formatter",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
