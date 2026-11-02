import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function SpriteDescriptionPackerTool(): ReactElement {
  return <PlannedTool description={"Builds a description sprite from a directory of individual icons."} />;
}
