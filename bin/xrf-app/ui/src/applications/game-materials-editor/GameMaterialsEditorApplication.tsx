import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function GameMaterialsEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit physical materials and their interaction pairs."} />;
}
