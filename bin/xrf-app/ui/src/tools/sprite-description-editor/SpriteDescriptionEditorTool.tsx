import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function SpriteDescriptionEditorTool(): ReactElement {
  return <PlannedTool description={"Inspects and edits the icons packed into a description sprite."} />;
}
