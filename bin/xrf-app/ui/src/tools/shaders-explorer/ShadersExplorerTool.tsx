import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function ShadersExplorerTool(): ReactElement {
  return <PlannedTool description={"Inspect render shaders and level-compiler shader definitions."} />;
}
