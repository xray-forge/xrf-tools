import { FactCheckIcon } from "@xrf/ui/icons/fact-check";

import { EToolGroupId, EToolId, EToolStatus, IToolDescriptor } from "@/core/routing/tool";
import { createToolDescriptor } from "@/core/routing/tool-descriptor";

import { LUA_VERIFIER_HELP } from "./help";

export const LUA_VERIFIER_TOOL: IToolDescriptor = createToolDescriptor(
  {
    description: "Check Lua scripts for syntax and compatibility problems",
    group: EToolGroupId.SCRIPTS,
    help: LUA_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EToolId.LUA_VERIFIER,
    label: "Lua verifier",
    path: "/lua-verifier",
    status: EToolStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
