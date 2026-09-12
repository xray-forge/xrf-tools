import { ReactElement } from "react";

import { SpawnEditorUnpackForm } from "./components/SpawnEditorUnpackForm";

/** Extract a packed spawn file into editable chunks. */
export function SpawnUnpackerApplication(): ReactElement {
  return <SpawnEditorUnpackForm />;
}
