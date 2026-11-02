import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function SoundsExplorerTool(): ReactElement {
  return <PlannedTool description={"Listen to sounds and inspect their X-Ray metadata."} />;
}
