import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function LevelViewerApplication(): ReactElement {
  return <PlannedApplication description={"Explore compiled locations with textures and inspection overlays."} />;
}
