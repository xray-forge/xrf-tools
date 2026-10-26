import { default as FormatAlignLeftIcon } from "@mui/icons-material/FormatAlignLeft";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { LUA_FORMATTER_HELP } from "./help";

export const LUA_FORMATTER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Format Lua scripts with consistent project conventions",
    group: EApplicationGroupId.SCRIPTS,
    help: LUA_FORMATTER_HELP,
    icon: <FormatAlignLeftIcon />,
    id: EApplicationId.LUA_FORMATTER,
    label: "Lua formatter",
    path: "/lua-formatter",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
