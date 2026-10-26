import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LevelEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit level geometry and scene objects."} />;
}
