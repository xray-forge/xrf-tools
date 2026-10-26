import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SoundsExplorerApplication(): ReactElement {
  return <PlannedApplication description={"Listen to sounds and inspect their X-Ray metadata."} />;
}
