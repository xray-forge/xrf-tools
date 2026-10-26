import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LevelCompilerApplication(): ReactElement {
  return <PlannedApplication description={"Compile editable level scenes into game-ready locations."} />;
}
