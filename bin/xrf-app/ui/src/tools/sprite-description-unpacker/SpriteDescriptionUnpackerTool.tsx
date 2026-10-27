import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function SpriteDescriptionUnpackerTool(): ReactElement {
  return <PlannedTool description={"Extracts the individual icons out of a description sprite."} />;
}
