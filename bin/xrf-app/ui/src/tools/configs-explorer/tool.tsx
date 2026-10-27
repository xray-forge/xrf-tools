import { FolderOpenIcon } from "@xrf/ui/icons/folder-open";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { CONFIGS_EXPLORER_HELP } from "./help";

export const CONFIGS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse LTX configuration files",
    group: EToolGroupId.CONFIGS,
    help: CONFIGS_EXPLORER_HELP,
    icon: <FolderOpenIcon />,
    id: EToolId.CONFIGS_EXPLORER,
    label: "Configs explorer",
    path: "/configs-explorer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
