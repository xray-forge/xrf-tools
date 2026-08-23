import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function InfoPortionsExplorerApplication(): ReactElement {
  return <PlannedApplication description={"Browses info portions and what gives, requires or revokes each of them."} />;
}
