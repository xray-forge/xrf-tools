import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SpriteDescriptionPackerApplication(): ReactElement {
  return <PlannedApplication description={"Builds a description sprite from a directory of individual icons."} />;
}
