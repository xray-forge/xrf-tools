import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LuaVerifierApplication(): ReactElement {
  return <PlannedApplication description={"Check Lua scripts for syntax and compatibility problems."} />;
}
