import { ReactElement } from "react";

import { PlannedApplication } from "@/core/shell/editor/PlannedApplication";

export function SoundEnvironmentsEditorApplication(): ReactElement {
  return <PlannedApplication description={"Edit acoustic environments and reverb presets."} />;
}
