import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function LevelAiCompilerTool(): ReactElement {
  return <PlannedTool description={"Build AI maps, level graphs and cross tables."} />;
}
