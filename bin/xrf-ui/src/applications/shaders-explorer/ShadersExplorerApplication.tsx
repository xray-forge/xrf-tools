import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function ShadersExplorerApplication(): ReactElement {
  return <PlannedApplication description={"Inspect render shaders and level-compiler shader definitions."} />;
}
