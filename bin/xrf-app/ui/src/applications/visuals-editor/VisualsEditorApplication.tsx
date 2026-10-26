import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function VisualsEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit model surfaces, skeletons and motion properties."} />;
}
