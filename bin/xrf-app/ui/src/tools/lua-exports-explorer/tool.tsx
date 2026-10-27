import { CodeIcon } from "@xrf/ui/icons/code";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LUA_EXPORTS_EXPLORER_HELP } from "./help";

export const LUA_EXPORTS_EXPLORER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Browse Lua script namespaces, functions and source locations",
    group: EToolGroupId.SCRIPTS,
    help: LUA_EXPORTS_EXPLORER_HELP,
    icon: <CodeIcon />,
    id: EToolId.LUA_EXPORTS_EXPLORER,
    label: "Lua exports explorer",
    path: "/lua-exports-explorer",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
