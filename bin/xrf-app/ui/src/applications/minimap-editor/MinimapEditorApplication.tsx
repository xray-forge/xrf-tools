import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function MinimapEditorApplication(): ReactElement {
  return <PlannedApplication description={"Align a level map image to its level and edit its world bounds."} />;
}
