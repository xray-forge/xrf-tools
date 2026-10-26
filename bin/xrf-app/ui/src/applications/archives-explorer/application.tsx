import { default as ArchiveIcon } from "@mui/icons-material/Archive";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { ARCHIVES_EXPLORER_KEYBIND_COMMANDS } from "./commands";
import { ARCHIVES_EXPLORER_HELP } from "./help";

export const ARCHIVES_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse game archives and loose gamedata files",
    group: EApplicationGroupId.ARCHIVES,
    help: ARCHIVES_EXPLORER_HELP,
    icon: <ArchiveIcon />,
    id: EApplicationId.ARCHIVES_EXPLORER,
    keybindCommands: ARCHIVES_EXPLORER_KEYBIND_COMMANDS,
    label: "Archives explorer",
    path: "/archives-explorer",
    status: EApplicationStatus.READY,
  },
  {
    load: () => import("./runtime"),
  }
);
