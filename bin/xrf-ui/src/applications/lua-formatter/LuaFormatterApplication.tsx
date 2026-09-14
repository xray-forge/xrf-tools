import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LuaFormatterApplication(): ReactElement {
  return <PlannedApplication description={"Format Lua scripts with consistent project conventions."} />;
}
