import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function ShadersEditorTool(): ReactElement {
  return <PlannedTool description={"Edit render and level-compiler shader libraries."} />;
}
