import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function LuaExportsExplorerTool(): ReactElement {
  return <PlannedTool description={"Browse Lua script namespaces, functions and source locations."} />;
}
