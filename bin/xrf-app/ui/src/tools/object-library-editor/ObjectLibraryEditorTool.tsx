import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/editor/PlannedTool";

export function ObjectLibraryEditorTool(): ReactElement {
  return <PlannedTool description={"Manage reusable source objects and their previews."} />;
}
