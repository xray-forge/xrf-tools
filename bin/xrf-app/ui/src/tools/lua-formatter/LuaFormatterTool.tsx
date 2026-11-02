import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function LuaFormatterTool(): ReactElement {
  return <PlannedTool description={"Format Lua scripts with consistent project conventions."} />;
}
