import { default as CodeIcon } from "@mui/icons-material/Code";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { LUA_EXPORTS_EXPLORER_HELP } from "./help";

export const LUA_EXPORTS_EXPLORER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Browse Lua script namespaces, functions and source locations",
    group: EApplicationGroupId.SCRIPTS,
    help: LUA_EXPORTS_EXPLORER_HELP,
    icon: <CodeIcon />,
    id: EApplicationId.LUA_EXPORTS_EXPLORER,
    label: "Lua exports explorer",
    path: "/lua-exports-explorer",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
