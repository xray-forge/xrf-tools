import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function ObjectLibraryEditorApplication(): ReactElement {
  return <PlannedApplication description={"Manage reusable source objects and their previews."} />;
}
