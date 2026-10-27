import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function LightAnimationsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit animated light colors and timing."} />;
}
