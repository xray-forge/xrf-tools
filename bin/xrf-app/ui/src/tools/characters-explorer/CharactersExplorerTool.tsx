import { ReactElement } from "react";

import { PlannedTool } from "@/core/shell/chrome/PlannedTool";

export function CharactersExplorerTool(): ReactElement {
  return (
    <PlannedTool description={"Browses character profiles beside the visual, voice and icon each one references."} />
  );
}
