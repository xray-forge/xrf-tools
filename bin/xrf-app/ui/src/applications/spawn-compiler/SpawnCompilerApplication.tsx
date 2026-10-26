import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SpawnCompilerApplication(): ReactElement {
  return <PlannedApplication description={"Build game spawn data from level sources and graphs."} />;
}
