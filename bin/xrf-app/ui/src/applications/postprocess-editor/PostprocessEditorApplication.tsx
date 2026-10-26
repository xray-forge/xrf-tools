import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function PostprocessEditorApplication(): ReactElement {
  return <PlannedApplication description={"Author postprocess effect curves and previews."} />;
}
