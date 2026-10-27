import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function LightAnimationsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit animated light colors and timing."} />;
}
