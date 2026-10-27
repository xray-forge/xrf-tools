import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function LevelEditorTool(): ReactElement {
  return <PlannedTool description={"Edit level geometry and scene objects."} />;
}
