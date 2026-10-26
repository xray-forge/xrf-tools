import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function GamedataComparerApplication(): ReactElement {
  return <PlannedApplication description={"Compare installations, mods and effective asset contents."} />;
}
