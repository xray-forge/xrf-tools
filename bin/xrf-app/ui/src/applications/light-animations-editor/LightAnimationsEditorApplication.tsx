import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LightAnimationsEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit animated light colors and timing."} />;
}
