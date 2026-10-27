import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function SoundsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit sound parameters and prepare game audio."} />;
}
