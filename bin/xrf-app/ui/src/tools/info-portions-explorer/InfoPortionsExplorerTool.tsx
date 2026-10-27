import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function InfoPortionsExplorerTool(): ReactElement {
  return <PlannedTool description={"Browses info portions and what gives, requires or revokes each of them."} />;
}
