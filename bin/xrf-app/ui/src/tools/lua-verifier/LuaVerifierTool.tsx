import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function LuaVerifierTool(): ReactElement {
  return <PlannedTool description={"Check Lua scripts for syntax and compatibility problems."} />;
}
