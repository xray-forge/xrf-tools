import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function LevelDecompilerTool(): ReactElement {
  return <PlannedTool description={"Reconstruct editable scenes from compiled game levels."} />;
}
