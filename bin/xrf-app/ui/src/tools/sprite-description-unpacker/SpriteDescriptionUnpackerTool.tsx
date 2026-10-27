import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function SpriteDescriptionUnpackerTool(): ReactElement {
  return <PlannedTool description={"Extracts the individual icons out of a description sprite."} />;
}
