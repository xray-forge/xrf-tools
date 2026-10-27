import { ArchiveIcon } from "@xrf/ui/icons/archive";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { ARCHIVES_EXPLORER_KEYBIND_COMMANDS } from "./commands";
import { ARCHIVES_EXPLORER_HELP } from "./help";

export const ARCHIVES_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse game archives and loose gamedata files",
    group: EToolGroupId.ARCHIVES,
    help: ARCHIVES_EXPLORER_HELP,
    icon: <ArchiveIcon />,
    id: EToolId.ARCHIVES_EXPLORER,
    keybindCommands: ARCHIVES_EXPLORER_KEYBIND_COMMANDS,
    label: "Archives explorer",
    path: "/archives-explorer",
    status: EToolStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
