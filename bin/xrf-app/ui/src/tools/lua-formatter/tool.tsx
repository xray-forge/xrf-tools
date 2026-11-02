import { FormatAlignLeftIcon } from "@xrf/ui/icons/format-align-left";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LUA_FORMATTER_HELP } from "./help";

export const LUA_FORMATTER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Format Lua scripts with consistent project conventions",
    group: EToolGroupId.SCRIPTS,
    help: LUA_FORMATTER_HELP,
    icon: <FormatAlignLeftIcon />,
    id: EToolId.LUA_FORMATTER,
    label: "Lua formatter",
    path: "/lua-formatter",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
