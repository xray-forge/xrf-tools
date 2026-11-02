import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function GameMaterialsEditorTool(): ReactElement {
  return <PlannedTool description={"Edit physical materials and their interaction pairs."} />;
}
