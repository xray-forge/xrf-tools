import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function GamedataComparerTool(): ReactElement {
  return <PlannedTool description={"Compare installations, mods and effective asset contents."} />;
}
