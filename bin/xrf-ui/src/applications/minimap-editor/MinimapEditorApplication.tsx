import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function MinimapEditorApplication(): ReactElement {
  return <PlannedApplication description={"Capture level minimaps and edit their world bounds."} />;
}
