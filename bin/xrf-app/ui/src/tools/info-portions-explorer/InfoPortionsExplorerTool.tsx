import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function InfoPortionsExplorerTool(): ReactElement {
  return <PlannedTool description={"Browses info portions and what gives, requires or revokes each of them."} />;
}
