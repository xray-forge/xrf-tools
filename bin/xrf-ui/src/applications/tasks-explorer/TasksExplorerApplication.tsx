import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function TasksExplorerApplication(): ReactElement {
  return <PlannedApplication description={"Browses quest tasks and the condlists deciding their titles and state."} />;
}
