import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function ShadersEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit render and level-compiler shader libraries."} />;
}
