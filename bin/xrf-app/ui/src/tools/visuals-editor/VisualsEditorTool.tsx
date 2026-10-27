import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function VisualsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit model surfaces, skeletons and motion properties."} />;
}
