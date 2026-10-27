import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function LevelEditorTool(): ReactElement {
  return <PlannedTool description={"Edit level geometry and scene objects."} />;
}
