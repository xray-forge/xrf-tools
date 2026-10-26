import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LevelDecompilerApplication(): ReactElement {
  return <PlannedApplication description={"Reconstruct editable scenes from compiled game levels."} />;
}
