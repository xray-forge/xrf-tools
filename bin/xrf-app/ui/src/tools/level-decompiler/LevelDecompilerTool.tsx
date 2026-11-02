import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function LevelDecompilerTool(): ReactElement {
  return <PlannedTool description={"Reconstruct editable scenes from compiled game levels."} />;
}
