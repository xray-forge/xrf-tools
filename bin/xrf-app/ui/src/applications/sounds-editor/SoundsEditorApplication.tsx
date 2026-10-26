import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SoundsEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit sound parameters and prepare game audio."} />;
}
