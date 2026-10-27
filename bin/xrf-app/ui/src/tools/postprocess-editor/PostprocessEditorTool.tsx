import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function PostprocessEditorTool(): ReactElement {
  return <PlannedTool description={"Author postprocess effect curves and previews."} />;
}
