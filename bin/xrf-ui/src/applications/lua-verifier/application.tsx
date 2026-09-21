import { default as FactCheckIcon } from "@mui/icons-material/FactCheck";

import {
  EApplicationGroupId,
  EApplicationId,
  EApplicationStatus,
  IApplicationDescriptor,
} from "@/core/routing/application";
import { createApplicationDescriptor } from "@/core/routing/application-descriptor";

import { LUA_VERIFIER_HELP } from "./help";

export const LUA_VERIFIER_APPLICATION: IApplicationDescriptor = createApplicationDescriptor(
  {
    description: "Check Lua scripts for syntax and compatibility problems",
    group: EApplicationGroupId.SCRIPTS,
    help: LUA_VERIFIER_HELP,
    icon: <FactCheckIcon />,
    id: EApplicationId.LUA_VERIFIER,
    label: "Lua verifier",
    path: "/lua-verifier",
    status: EApplicationStatus.PLANNED,
  },
  {
    load: () => import("./runtime"),
  }
);
