import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function ParticlesExplorerTool(): ReactElement {
  return <PlannedTool description={"Browse particle effects, groups and dependencies."} />;
}
