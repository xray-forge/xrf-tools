import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function SoundsExplorerTool(): ReactElement {
  return <PlannedTool description={"Listen to sounds and inspect their X-Ray metadata."} />;
}
