import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function MinimapEditorTool(): ReactElement {
  return <PlannedTool description={"Align a level map image to its level and edit its world bounds."} />;
}
