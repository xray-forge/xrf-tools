import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function ParticlesEditorTool(): ReactElement {
  return <PlannedTool description={"Author particle effects and groups with a live preview."} />;
}
