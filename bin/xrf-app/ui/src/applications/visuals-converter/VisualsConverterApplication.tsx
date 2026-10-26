import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function VisualsConverterApplication(): ReactElement {
  return <PlannedApplication description={"Convert models and animations between game and editable formats."} />;
}
