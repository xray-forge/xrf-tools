import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function LuaExportsExplorerTool(): ReactElement {
  return <PlannedTool description={"Browse Lua script namespaces, functions and source locations."} />;
}
