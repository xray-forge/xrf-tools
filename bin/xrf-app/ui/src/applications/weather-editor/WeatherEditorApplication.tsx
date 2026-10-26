import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function WeatherEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit weather cycles, effects and environment settings."} />;
}
