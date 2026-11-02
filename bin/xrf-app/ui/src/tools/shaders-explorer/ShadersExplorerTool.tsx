import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function ShadersExplorerTool(): ReactElement {
  return <PlannedTool description={"Inspect render shaders and level-compiler shader definitions."} />;
}
