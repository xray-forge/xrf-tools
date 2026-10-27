import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function MinimapEditorTool(): ReactElement {
  return <PlannedTool description={"Align a level map image to its level and edit its world bounds."} />;
}
