import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LuaExportsExplorerApplication(): ReactElement {
  return <PlannedApplication description={"Browse Lua script namespaces, functions and source locations."} />;
}
