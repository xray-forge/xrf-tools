import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function CharactersExplorerApplication(): ReactElement {
  return (
    <PlannedApplication
      description={"Browses character profiles beside the visual, voice and icon each one references."}
    />
  );
}
