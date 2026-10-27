import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function SpriteEquipmentUnpackerTool(): ReactElement {
  return <PlannedTool description={"Extracts the individual icons out of an equipment sprite."} />;
}
