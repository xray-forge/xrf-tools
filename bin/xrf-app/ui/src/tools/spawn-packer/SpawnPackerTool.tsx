import { ReactElement } from "react";

import { SpawnEditorPackForm } from "./components/SpawnEditorPackForm";

/** Build a packed spawn file out of unpacked chunks. */
export function SpawnPackerTool(): ReactElement {
  return <SpawnEditorPackForm />;
}
