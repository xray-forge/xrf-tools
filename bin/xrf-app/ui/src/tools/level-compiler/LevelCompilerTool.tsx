import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function LevelCompilerTool(): ReactElement {
  return <PlannedTool description={"Compile editable level scenes into game-ready locations."} />;
}
