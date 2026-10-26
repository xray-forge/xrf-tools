import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SpriteEquipmentUnpackerApplication(): ReactElement {
  return <PlannedApplication description={"Extracts the individual icons out of an equipment sprite."} />;
}
