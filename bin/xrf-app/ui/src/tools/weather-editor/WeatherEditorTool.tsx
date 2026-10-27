import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function WeatherEditorTool(): ReactElement {
  return <PlannedTool description={"Edit weather cycles, effects and environment settings."} />;
}
