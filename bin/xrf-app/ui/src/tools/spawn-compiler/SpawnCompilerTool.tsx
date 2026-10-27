import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function SpawnCompilerTool(): ReactElement {
  return <PlannedTool description={"Build game spawn data from level sources and graphs."} />;
}
