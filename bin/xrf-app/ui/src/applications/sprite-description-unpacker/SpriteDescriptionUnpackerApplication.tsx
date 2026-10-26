import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SpriteDescriptionUnpackerApplication(): ReactElement {
  return <PlannedApplication description={"Extracts the individual icons out of a description sprite."} />;
}
