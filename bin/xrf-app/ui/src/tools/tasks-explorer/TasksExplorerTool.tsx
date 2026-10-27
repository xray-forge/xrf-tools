import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function TasksExplorerTool(): ReactElement {
  return <PlannedTool description={"Browses quest tasks and the condlists deciding their titles and state."} />;
}
