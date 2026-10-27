import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function VisualsConverterTool(): ReactElement {
  return <PlannedTool description={"Convert models and animations between game and editable formats."} />;
}
