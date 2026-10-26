import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function ParticlesExplorerApplication(): ReactElement {
  return <PlannedApplication description={"Browse particle effects, groups and dependencies."} />;
}
